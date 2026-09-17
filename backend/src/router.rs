//! Extensible routing of incoming MQTT messages to handlers

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rumqttc::{AsyncClient, QoS};
use sqlx::SqlitePool;

use shared::mqtt_convention;
use shared::packets::{Packet as NodePacket, PacketPayload};

use crate::db::entries::{NodeEntry, NodeTelemetryEntry};

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub type Handler = Arc<dyn Fn(String, Vec<u8>, Arc<AsyncClient>, SqlitePool) -> BoxFuture + Send + Sync>;

/// All active subscriptions: (filter, handler) pairs
pub fn routes() -> Vec<(String, Handler)> {
    // Node telemetry
    let mut telemetry_filter = String::new();
    mqtt_convention::node_telemetry(&mut telemetry_filter, "+");

    // Node boot
    let mut node_boot_filter = String::new();
    mqtt_convention::node_boot(&mut node_boot_filter, "+");

    vec![
        (telemetry_filter, handler(handle_telemetry)),
        (node_boot_filter, handler(handle_boot)),
    ]
}

/// Wraps a plain async fn into the boxed-future form the route table needs
fn handler<F, Fut>(f: F) -> Handler
where
    F: Fn(String, Vec<u8>, Arc<AsyncClient>, SqlitePool) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Arc::new(move |topic, payload, client, pool| Box::pin(f(topic, payload, client, pool)))
}

/// Find the first route whose filter matches `topic` and run its handler
pub async fn dispatch(
    routes: &[(String, Handler)],
    topic: &str,
    payload: &[u8],
    client: &Arc<AsyncClient>,
    pool: &SqlitePool,
) {
    for (filter, handle) in routes {
        if topic_matches(filter, topic) {
            handle(topic.to_string(), payload.to_vec(), client.clone(), pool.clone()).await;
            return;
        }
    }
    eprintln!("No handler registered for topic: {topic}");
}

/// MQTT topic-filter matching (`+` = one level, `#` = rest of topic).
fn topic_matches(filter: &str, topic: &str) -> bool {
    let filter_levels: Vec<&str> = filter.split('/').collect();
    let topic_levels: Vec<&str> = topic.split('/').collect();

    for (i, f) in filter_levels.iter().enumerate() {
        if *f == "#" {
            return true;
        }
        match topic_levels.get(i) {
            Some(t) if *f == "+" || f == t => continue,
            _ => return false,
        }
    }
    filter_levels.len() == topic_levels.len()
}

/// Helper function to override settings
async fn override_settings(client: Arc<AsyncClient>, pool: &SqlitePool, entry: NodeEntry) {
    let mut t = String::new();
    mqtt_convention::settings_override(&mut t, &*entry.node_id);

    let settings = entry.get_settings(&pool).await;
    if settings.is_err() {
        eprintln!("DB Error: {:?}", settings.err());
        return;
    }
    let settings = settings.unwrap();

    let p = NodePacket::new(0, PacketPayload::SettingsOverride(settings));
    let buf = &mut [0u8; 512];

    match p.serialize(buf) {
        Ok(len) => match client.publish(t.clone(), QoS::AtMostOnce, false, &buf[..len]).await {
            Ok(_) => println!("SettingsOverride packet published ({})", t),
            Err(e) => eprintln!("Failed to publish SettingsOverride packet: {:?}", e),
        },
        Err(e) => eprintln!("Failed to serialize SettingsOverride packet: {:?}", e),
    };
}

/// Telemetry handler
async fn handle_telemetry(topic: String, payload: Vec<u8>, _client: Arc<AsyncClient>, pool: SqlitePool) {
    let node_id = topic.split('/').nth(1).unwrap();
    match NodePacket::deserialize(&payload) {
        Ok(packet) => {
            println!("Telemetry from {topic}:\n{packet:#?}") ;
            match packet.payload {
                PacketPayload::Telemetry(telemetry) => {
                    match NodeTelemetryEntry::push(&pool, node_id, packet.header.uptime_ms as i64, telemetry).await {
                        Ok(_) => {},
                        Err(e) => eprintln!("Error pushing telemetry: {e}")
                    };
                }
                _ => eprintln!("Invalid telemetry packet payload!")
            }
        },
        Err(err) => eprintln!("Failed to deserialize packet from {topic}: {err:?}"),
    }
}

/// Node boot handler
async fn handle_boot(topic: String, payload: Vec<u8>, client: Arc<AsyncClient>, pool: SqlitePool) {
    match NodePacket::deserialize(&payload) {
        Ok(packet) => {
            println!("Node boot notification: {topic}:\n{packet:#?}");
            let config = match packet.payload {
                PacketPayload::NodeBoot(config) => config,
                _ => {
                    eprintln!("Invalid NodeBoot packet payload!");
                    return;
                }
            };
            
            let node_id = topic.split('/').nth(1).unwrap();
            
            match NodeEntry::from_node_id(&pool, node_id).await {
                // Node entry already exists
                Ok(Some(mut entry)) => {
                    match entry.update_boot(&pool, packet.header.uptime_ms).await {
                        Ok(_) => println!("Boot updated for node {node_id}"),
                        Err(_) => println!("Failed to update boot for node {node_id}"),
                    }
                    if entry.get_config() != config {
                        match entry.update_config(&pool, config).await {
                            Ok(_) => println!("Config updated for node {node_id}"),
                            Err(_) => eprintln!("Failed to update config for node {node_id}"),
                        }
                    }

                    override_settings(client, &pool, entry).await;
                }
                
                // Node entry does not exist yet
                Ok(None) => {
                    let res = NodeEntry::push_default(&pool, node_id, packet.header.uptime_ms, config).await;
                    if res.is_err() {
                        eprintln!("DB Error: {}", res.err().unwrap());
                        return;
                    }

                    // Force the initialization of channels
                    let entry = res.unwrap();
                    override_settings(client, &pool, entry).await;
                }
                
                // faaah
                Err(e) => {
                    eprintln!("DB error: {e}");
                    return;
                }
            };
        }
        Err(err) => eprintln!("Failed to deserialize packet from {topic}: {err:?}"),
    }
}