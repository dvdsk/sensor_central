#[cfg(feature = "ble")]
use bluebus;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "ble")]
    Ble(bluebus::Error),
    #[cfg(feature = "local")]
    GPIO(gpio_cdev::errors::Error),
    #[cfg(feature = "local")]
    GPIONotFound,
}

#[cfg(feature = "local")]
impl From<gpio_cdev::errors::Error> for Error {
    fn from(error: gpio_cdev::errors::Error) -> Self {
        Error::GPIO(error)
    }
}

#[cfg(feature = "ble")]
impl From<bluebus::Error> for Error {
    fn from(error: bluebus::Error) -> Self {
        Error::Ble(error)
    }
}