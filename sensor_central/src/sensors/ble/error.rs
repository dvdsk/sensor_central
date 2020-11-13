use bluebus::Error::*;
use bluebus::rustbus::client_conn::Error as RustbusError;
use std::mem::discriminant;

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
            | NoFdReturned
            | InvalidLength(_) => ConnectionError::Unrecoverable(err),

            DbusConnectionError(ref e) if discriminant(e) == discriminant(&RustbusError::TimedOut) => {
                ConnectionError::Recoverable(err)
            }
            
            CouldNotConnectToBus(_)
            | CharacteristicNotFound(_) //Yeah this happens even if the characteristic is present and can be recovered 
            | AuthenticationCanceled(_)
            | BluezFailed(_) 
            | CouldNotConnectToDevice 
            | InProgress(_)
            | PairingTimeOut => {
                ConnectionError::Recoverable(err)
            }

            _ => panic!("{:?}, should not occur during connection", err),
        }
    }
}
