#[derive(Debug)]
pub enum Error {
    GPIO(gpio_cdev::errors::Error),
    GPIONotFound,
}

impl From<gpio_cdev::errors::Error> for Error {
    fn from(error: gpio_cdev::errors::Error) -> Self {
        Error::GPIO(error)
    }
}