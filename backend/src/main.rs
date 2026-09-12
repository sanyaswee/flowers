//! The main backend server

use std::sync::Arc;
use std::time::Duration;

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet as MqttPacket, QoS};

mod router;

#[tokio::main]
async fn main() {
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
    println!("Listening for MQTT messages on 127.0.0.1:1883");

    poll_loop(event_loop, routes, client).await;
}

/// MQTT event loop. Each incoming publish is dispatched on its own task
async fn poll_loop(
    mut event_loop: EventLoop,
    routes: Arc<Vec<(String, router::Handler)>>,
    client: Arc<AsyncClient>,
) {
    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(MqttPacket::Publish(publish))) => {
                let routes = routes.clone();
                let client = client.clone();
                tokio::spawn(async move {
                    router::dispatch(&routes, &publish.topic, &publish.payload, &client).await;
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