#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use cyw43::aligned_bytes;
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use embassy_executor::Spawner;
use embassy_net::{Config as NetConfig, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::dma;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

const WIFI_SSID: &str = include_str!("../../secrets/ssid.txt");
const WIFI_PASSWORD: &str = include_str!("../../secrets/password.txt");

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Firmware blobs, 4-byte aligned via cyw43's own macro (don't build `Aligned` by hand).
    // Grab nvram_rp2040.bin from:
    // https://github.com/embassy-rs/embassy/raw/main/cyw43-firmware/nvram_rp2040.bin
    // and place it alongside 43439A0.bin / 43439A0_clm.bin.
    let fw = aligned_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = aligned_bytes!("../cyw43-firmware/43439A0_clm.bin");
    let nvram = aligned_bytes!("../cyw43-firmware/nvram_rp2040.bin");

    // --- Wi-Fi Hardware Setup ---
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        dma::Channel::new(p.DMA_CH0, Irqs),
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());

    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;
    spawner.spawn(cyw43_task(runner).unwrap());

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // --- Network Stack Setup ---
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, net_runner) = embassy_net::new(
        net_device,
        NetConfig::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        0x0123_4567_89ab_cdef, // TODO: replace with a TRNG-derived seed in production
    );
    spawner.spawn(net_task(net_runner).unwrap());

    // --- Connect to Wi-Fi ---
    info!("Connecting to Wi-Fi network: {}...", WIFI_SSID);
    loop {
        match control
            .join(WIFI_SSID, cyw43::JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("Join failed with error: {}. Retrying...", err);
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }
    info!("Wi-Fi connected! Waiting for DHCP lease...");

    stack.wait_config_up().await;
    if let Some(config) = stack.config_v4() {
        let ip = config.address.address().octets();
        info!("Network configured! IP address: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    }

    info!("Node initialized and network connected");
}