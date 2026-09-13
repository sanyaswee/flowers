//! This module contains `TcpProvider` trait that should be implemented by nodes for proper Wi-Fi usage

use embedded_io_async::{Read, Write};

/// Should be implemented inside node specific crate
pub trait TcpProvider {
    type Stream: Read + Write;
    type Error: defmt::Format;

    /// Establish a raw TCP connection to the broker
    async fn connect(&mut self) -> Result<Self::Stream, Self::Error>;
}