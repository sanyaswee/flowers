#![no_std]
#![no_main]

mod wifi;

use defmt_rtt as _;
use panic_probe as _;

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{Async, Config, I2c, InterruptHandler as I2cInterruptHandler};
use embassy_rp::peripherals;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use static_cell::make_static; // The macro that removes the bloat

use cyw43_pio::PioSpi;
use embassy_net::{Config as NetConfig, Stack, StackResources};

use core_logic::i2c_mutex::SharedI2C;
use core_logic::{bh1750, bmp280, telemetry_broker};
use crate::wifi::WifiCredentials;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2cInterruptHandler<peripherals::I2C0>;
    PIO0_IRQ_0 => PioInterruptHandler<peripherals::PIO0>;
});

type PicoI2c = I2c<'static, peripherals::I2C0, Async>;

const WIFI: WifiCredentials = WifiCredentials {
    ssid: include_str!("../../secrets/ssid.txt"),
    password: include_str!("../../secrets/password.txt"),
};

// Use cyw43's built-in aligner so we don't need manual struct wrappers or unsafe code
static FW: cyw43::Aligned<u32, [u8; 231077]> = cyw43::Aligned(*include_bytes!("../../cyw43-firmware/43439A0.bin"));
static CLM: cyw43::Aligned<u32, [u8; 984]> = cyw43::Aligned(*include_bytes!("../../cyw43-firmware/43439A0_clm.bin"));

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        cyw43::SpiBus<Output<'static>, PioSpi<'static, peripherals::PIO0, 0>>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn read_light_intensity(bus: &'static SharedI2C<PicoI2c>) {
    bh1750::read_light_intensity(bus).await;
}

#[embassy_executor::task]
async fn read_temp_pressure(bus: &'static SharedI2C<PicoI2c>) {
    bmp280::read_temp_pressure(bus).await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // --- I2C Setup ---
    let i2c = I2c::new_async(p.I2C0, p.PIN_17, p.PIN_16, Irqs, Config::default());
    let shared_i2c = make_static!(SharedI2C<PicoI2c>, Mutex::new(i2c));

    // --- Wi-Fi Hardware Setup ---
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        Default::default(),
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0.into(),
    );

    let state = make_static!(cyw43::State, cyw43::State::new());

    // Pass references to the aligned arrays directly
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, &FW.0).await;
    spawner.spawn(wifi_task(runner).unwrap());

    control.init(&CLM.0).await;
    control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

    // --- Network Stack Setup ---
    let res = make_static!(StackResources<3>, StackResources::new());
    let (stack, net_runner) = embassy_net::new(
        net_device,
        NetConfig::dhcpv4(Default::default()),
        res,
        0x0123_4567_89ab_cdef, // Hardcoded seed, replace with TRNG in production
    );
    spawner.spawn(net_task(net_runner).unwrap());

    let stack = make_static!(Stack<'static>, stack);

    // --- Connect to Wi-Fi ---
    info!("Connecting to Wi-Fi network: {}...", WIFI.ssid);
    loop {
        match control.join(WIFI.ssid, cyw43::JoinOptions::new(WIFI.password.as_bytes())).await {
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
        let ip = config.address.address().0;
        info!("Network configured! IP address: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    }

    // --- Spawn Application Tasks ---
    spawner.spawn(read_light_intensity(shared_i2c).unwrap());
    spawner.spawn(read_temp_pressure(shared_i2c).unwrap());
    spawner.spawn(telemetry_broker::publish().unwrap());

    info!("Node initialized and network connected");
}