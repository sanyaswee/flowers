#![no_std]
#![no_main]

mod wifi;

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let stack = wifi::init(
        spawner, p.PIO0, p.PIN_23, p.PIN_24, p.PIN_25, p.PIN_29, p.DMA_CH0,
    )
        .await;

    if let Some(config) = stack.config_v4() {
        let ip = config.address.address().octets();
        info!(
            "Network configured! IP address: {}.{}.{}.{}",
            ip[0], ip[1], ip[2], ip[3]
        );
    }

    info!("Node initialized!");
}