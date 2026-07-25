//! Wi-Fi driver for the CYW43439 chip on the Pico 2 W.
//!
//! Calling `wifi::init(...)` once from `main` spawns the driver + network tasks

use cyw43::aligned_bytes;
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{Config as NetConfig, Stack, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::dma;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::Peri;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

/// Load Wi-Fi credentials
/// TODO replace with AP later
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

/// Initialize the CYW43 chip, join the network and wait for IP
pub async fn init(
    spawner: Spawner,
    pio0: Peri<'static, PIO0>,
    pin_23: Peri<'static, PIN_23>,
    pin_24: Peri<'static, PIN_24>,
    pin_25: Peri<'static, PIN_25>,
    pin_29: Peri<'static, PIN_29>,
    dma_ch0: Peri<'static, DMA_CH0>,
) -> Stack<'static> {
    // Load CYW43 firmware
    // Taken from: https://github.com/embassy-rs/embassy/raw/main/cyw43-firmware/
    let fw = aligned_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = aligned_bytes!("../cyw43-firmware/43439A0_clm.bin");
    let nvram = aligned_bytes!("../cyw43-firmware/nvram_rp2040.bin");

    // Hardware setup
    let pwr = Output::new(pin_23, Level::Low);
    let cs = Output::new(pin_25, Level::High);
    let mut pio = Pio::new(pio0, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        pin_24,
        pin_29,
        dma::Channel::new(dma_ch0, Irqs),
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());

    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;
    spawner.spawn(cyw43_task(runner).unwrap());

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Network setup
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, net_runner) = embassy_net::new(
        net_device,
        NetConfig::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        0x0123_4567_89ab_cdef, // TODO: replace with a TRNG seed
    );
    spawner.spawn(net_task(net_runner).unwrap());

    // Connect to Wi-Fi
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

    stack
}