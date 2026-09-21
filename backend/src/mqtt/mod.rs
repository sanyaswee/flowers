//! The MQTT module

use std::sync::Arc;
use std::time::Duration;

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet as MqttPacket, QoS};
use sqlx::SqlitePool;

pub mod router;

/// Initialize the MQTT task
pub async fn init(pool: SqlitePool) -> Arc<AsyncClient> {
    let mut mqtt_options = MqttOptions::new("flowers-backend", "127.0.0.1", 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(60));

    let (client, event_loop) = AsyncClient::new(mqtt_options, 100);
    let client = Arc::new(client);

    let routes = Arc::new(router::routes());

    for (filter, _) in routes.iter() {
        client
            .subscribe(filter, QoS::AtMostOnce)
            .await
            .expect("Failed to subscribe to topic");
        println!("Subscribed to {filter}");
    }

    // Publish ServerBoot packet
    router::publish_boot(&client).await;

    // Spawn listener task and return a client
    println!("Listening for MQTT messages on 127.0.0.1:1883");
    let client_ = client.clone();
    tokio::spawn(async move {
        poll_loop(event_loop, routes, client, pool).await;
    });
    client_
}

/// MQTT event loop. Each incoming publish is dispatched on its own task
async fn poll_loop(
    mut event_loop: EventLoop,
    routes: Arc<Vec<(String, router::Handler)>>,
    client: Arc<AsyncClient>,
    pool: SqlitePool,
) {
    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(MqttPacket::Publish(publish))) => {
                let routes = routes.clone();
                let client = client.clone();
                let pool = pool.clone(); // safe because SqlitePool is backed by Arc inside sqlx
                tokio::spawn(async move {
                    router::dispatch(&routes, &publish.topic, &publish.payload, &client, &pool)
                        .await;
                });
            }
            Ok(_) => {
                // Ignore protocol control packets
            }
            Err(e) => {
                eprintln!("Broker connection error: {e:?}");
                // Give the broker a moment before rumqttc tries to reconnect
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
