//! The main backend server

use rumqttc::{Client, Event, MqttOptions, Packet as MqttPacket, QoS};
use std::time::Duration;

use shared::packets::Packet as NodePacket;

fn main() {
    // Connect to local broker
    println!("Packet size: {} bytes", size_of::<NodePacket>());
    let mut mqtt_options = MqttOptions::new("backend-subscriber", "127.0.0.1", 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(60));

    // Initialize the MQTT client with a channel capacity of 100
    let (mut client, mut connection) = Client::new(mqtt_options, 100);

    // Subscribe to all node telemetry topics
    client
        .subscribe("node/+/telemetry", QoS::AtMostOnce)
        .expect("Failed to subscribe to topic");

    println!("Listening for MQTT messages on 127.0.0.1:1883...");

    // connection.iter() loop automatically handles reconnection and polling
    for notification in connection.iter() {
        match notification {
            Ok(Event::Incoming(MqttPacket::Publish(publish))) => {
                // Pass the raw byte payload into existing deserialization logic
                match NodePacket::deserialize(&publish.payload) {
                    Ok(packet) => {
                        println!("Received from {}:\n{packet:#?}", publish.topic);
                    }
                    Err(err) => {
                        eprintln!("Failed to deserialize packet from {}: {err:?}", publish.topic);
                    }
                }
            }
            Ok(_) => {
                // Ignore PINGRESP, SUBACK, and other protocol control packets
            }
            Err(e) => {
                eprintln!("Broker connection error: {e:?}");
                // rumqttc automatically attempts to reconnect in the background
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }
}