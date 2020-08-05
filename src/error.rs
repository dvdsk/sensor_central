use bluebus;

#[derive(Debug)]
pub enum Error {
    Ble(bluebus::Error),
    GPIO(gpio_cdev::errors::Error),
    GPIONotFound,
}

impl From<gpio_cdev::errors::Error> for Error {
    fn from(error: gpio_cdev::errors::Error) -> Self {
        Error::GPIO(error)
    }
}

impl From<bluebus::Error> for Error {
    fn from(error: bluebus::Error) -> Self {
        Error::Ble(error)
    }
}