use bluebus::Error::*;

#[derive(Debug)]
pub enum ConnectionError {
    Recoverable(bluebus::Error),
    Unrecoverable(bluebus::Error),
}

impl ConnectionError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            ConnectionError::Recoverable(_) => true,
            ConnectionError::Unrecoverable(_) => false,
        }
    }
}

impl From<bluebus::Error> for ConnectionError {
    fn from(err: bluebus::Error) -> Self {
        match err {
            AuthenticationFailed(_)
            | UuidNotFound
            | CharacteristicNotFound(_)
            | NoFdReturned
            | InvalidLength(_) => ConnectionError::Unrecoverable(err),

            CouldNotConnectToBus(_) | BluezFailed(_) | CouldNotConnectToDevice => {
                ConnectionError::Recoverable(err)
            }

            _ => panic!("{:?}, should not occur during connection", err),
        }
    }
}
