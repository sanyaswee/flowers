//! A mutex for the shared I2C bus

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;

pub type SharedI2C<I2C> = Mutex<NoopRawMutex, I2C>;