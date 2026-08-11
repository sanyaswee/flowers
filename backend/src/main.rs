//! The host server

use std::io::Read;
use std::net::TcpListener;

use shared::packets::Packet;

/// Address to listen
/// TODO dynamically allocate later
const ADDR: &str = "0.0.0.0:8000";

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDR);
    println!("Listening on {}", ADDR);

    for conn in listener?.incoming() {
        let mut stream = conn?;
        println!("Node connected: {}", stream.peer_addr()?);

        let mut buffer = [0u8; 1024];

        loop {
            let n = stream.read(&mut buffer)?;

            if n == 0 {
                println!("Node disconnected");
                break;
            }

            match Packet::deserialize(&buffer[..n]) {
                Ok(packet) => {
                    println!("Received packet: {packet:#?}");
                }
                Err(err) => {
                    eprintln!("Failed to deserialize packet: {err:?}");
                }
            }
        }
    }

    Ok(())
}
