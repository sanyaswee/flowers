use defmt::Format;

#[derive(Format)]
pub struct WifiCredentials {
    pub ssid: &'static str,
    pub password: &'static str,
}