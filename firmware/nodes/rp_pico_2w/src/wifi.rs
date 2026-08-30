//! Wi-Fi driver for the CYW43439 chip on the Pico 2 W
//!
//! `wifi::init(...)` spawns the driver + network tasks

use core::str::FromStr;

use cyw43::aligned_bytes;
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};

use defmt::info;

use embassy_executor::Spawner;
use embassy_net::tcp::{ConnectError, Error as TcpError, TcpSocket, State};
use embassy_net::{Config as NetConfig, IpEndpoint, Ipv4Address, Stack, StackResources};

use embassy_rp::bind_interrupts;
use embassy_rp::dma;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::Peri;

use embassy_time::{Duration, Timer};

use static_cell::StaticCell;

use core_logic::network::PacketSender;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

/// Load Wi-Fi credentials
/// TODO replace with AP later
const WIFI_SSID: &str = include_str!("../../../secrets/ssid.txt");
const WIFI_PASSWORD: &str = include_str!("../../../secrets/password.txt");

/// Get server address
/// TODO find a better way to handle this
const SERVER_IP: &str = include_str!("../../../secrets/ip.txt");
const SERVER_PORT: u16 = 8000;

/// Buffer sizes for the TCP socket.
const TCP_RX_BUFFER_SIZE: usize = 256;
const TCP_TX_BUFFER_SIZE: usize = 256;

/// Actual TCP socket buffers
static RX_BUFFER: StaticCell<[u8; TCP_RX_BUFFER_SIZE]> = StaticCell::new();
static TX_BUFFER: StaticCell<[u8; TCP_TX_BUFFER_SIZE]> = StaticCell::new();

/// Errors that can occur while sending data over the Wi-Fi
#[derive(Debug)]
pub enum TransportError {
    /// Failed to open the TCP connection to the server
    Connect(ConnectError),

    /// Failed while writing to an established connection
    Io(TcpError),

    /// send() called while not connected
    NotConnected,
}

impl defmt::Format for TransportError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            TransportError::Connect(_) => defmt::write!(fmt, "TransportError::Connect"),
            TransportError::Io(_) => defmt::write!(fmt, "TransportError::Io"),
            TransportError::NotConnected => defmt::write!(fmt, "TransportError::NotConnected"),
        }
    }
}

/// The Wi-Fi transporter task
pub struct WifiTransport {
    pub stack: Stack<'static>,
    server: IpEndpoint,
    socket: TcpSocket<'static>,
}

impl PacketSender for WifiTransport {
    type Error = TransportError;

    async fn send(&mut self, topic: &str, bytes: &[u8]) -> Result<(), Self::Error> {
        if self.socket.state() != State::Established {
            return Err(TransportError::NotConnected);
        }

        let mut written = 0;
        while written < bytes.len() {
            let n = self.socket.write(&bytes[written..]).await.map_err(|e| {
                // Write failed mid-stream: force the socket closed so the
                // next reconnect() attempt starts clean rather than being
                // fooled by a half-dead state.
                self.socket.abort();
                TransportError::Io(e)
            })?;
            written += n;
        }
        self.socket.flush().await.map_err(TransportError::Io)
    }

    async fn reconnect(&mut self) -> Result<(), Self::Error> {
        if self.socket.state() == State::Established {
            return Ok(());
        }
        self.socket.abort(); // clean slate in case it's in some half-open state
        self.socket
            .connect(self.server)
            .await
            .map_err(TransportError::Connect)
    }
}

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
) -> WifiTransport {
    // Load CYW43 firmware
    // Taken from: https://github.com/embassy-rs/embassy/raw/main/cyw43-firmware/
    let fw = aligned_bytes!("../../../cyw43/43439A0.bin");
    let clm = aligned_bytes!("../../../cyw43/43439A0_clm.bin");
    let nvram = aligned_bytes!("../../../cyw43/nvram_rp2040.bin");

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
        .set_power_management(cyw43::PowerManagementMode::None)
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

    let server_addr = Ipv4Address::from_str(SERVER_IP.trim())
        .expect("SERVER_IP in secrets/server_ip.txt is not a valid IPv4 address");
    let server = IpEndpoint::from((server_addr, SERVER_PORT));

    info!("Server target set to {}:{}", SERVER_IP, SERVER_PORT);

    let rx_buffer = RX_BUFFER.init([0u8; TCP_RX_BUFFER_SIZE]);
    let tx_buffer = TX_BUFFER.init([0u8; TCP_TX_BUFFER_SIZE]);
    let socket = TcpSocket::new(stack, rx_buffer, tx_buffer);

    WifiTransport { stack, server, socket }
}