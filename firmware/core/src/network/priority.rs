//! Priority wrapper

use core::cmp::Ordering;
use shared::packets::{Packet, PacketPayload};

/// The wrapper struct for the packet so it can be sorted based on priority
pub struct PriorityPacketWrapper(pub Packet);

impl PriorityPacketWrapper {
    fn priority(&self) -> u8 {
        match &self.0.payload {
            // Boot packets have priority over telemetry packets
            PacketPayload::NodeBoot(_) => 2,
            PacketPayload::Telemetry(_) => 1,
            _ => 0,
        }
    }
}

impl PartialEq for PriorityPacketWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.priority() == other.priority()
    }
}

impl Eq for PriorityPacketWrapper {}

impl PartialOrd for PriorityPacketWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityPacketWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}
