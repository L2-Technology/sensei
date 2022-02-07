use std::fmt::{self, Display};

use crate::database;

#[derive(Debug)]
pub enum Error {
    Db(database::Error),
    TinderCrypt(tindercrypt::errors::Error),
    Macaroon(macaroon::MacaroonError),
    Io(std::io::Error),
    Secp256k1(bitcoin::secp256k1::Error),
    Bdk(bdk::Error),
    BdkLdk(bdk_ldk::Error),
    LdkApi(lightning::util::errors::APIError),
    LdkMsg(lightning::ln::msgs::LightningError),
    LdkInvoice(lightning_invoice::payment::PaymentError),
    LdkInvoiceSign(lightning_invoice::SignOrCreationError),
    LdkInvoiceParse(lightning_invoice::ParseOrSemanticError),
    InvalidSeedLength,
    FailedToWriteSeed,
    Unauthenticated,
    InvalidMacaroon,
    AdminNodeNotStarted,
    AdminNodeNotCreated,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Error::Db(e) => match e {
                database::Error::Generic(str) => str.clone(),
            },
            Error::Macaroon(_e) => "macaroon error".to_string(),
            Error::TinderCrypt(e) => e.to_string(),
            Error::Io(e) => e.to_string(),
            Error::Secp256k1(e) => e.to_string(),
            Error::Bdk(e) => e.to_string(),
            Error::BdkLdk(e) => match e {
                bdk_ldk::Error::Bdk(e) => e.to_string(),
            },
            Error::LdkApi(e) => format!("{:?}", e),
            Error::LdkMsg(e) => format!("{:?}", e),
            Error::LdkInvoice(e) => format!("{:?}", e),
            Error::LdkInvoiceSign(e) => e.to_string(),
            Error::LdkInvoiceParse(e) => e.to_string(),
            Error::InvalidSeedLength => String::from("invalid seed length"),
            Error::FailedToWriteSeed => String::from("failed to write seed"),
            Error::Unauthenticated => String::from("unauthenticated"),
            Error::InvalidMacaroon => String::from("invalid macaroon"),
            Error::AdminNodeNotCreated => String::from("admin node not created"),
            Error::AdminNodeNotStarted => String::from("admin node not started"),
        };
        write!(f, "{}", str)
    }
}

impl From<bitcoin::secp256k1::Error> for Error {
    fn from(e: bitcoin::secp256k1::Error) -> Error {
        Error::Secp256k1(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<bdk::Error> for Error {
    fn from(e: bdk::Error) -> Error {
        Error::Bdk(e)
    }
}

impl From<lightning_invoice::payment::PaymentError> for Error {
    fn from(e: lightning_invoice::payment::PaymentError) -> Self {
        Error::LdkInvoice(e)
    }
}

impl From<lightning_invoice::SignOrCreationError> for Error {
    fn from(e: lightning_invoice::SignOrCreationError) -> Self {
        Error::LdkInvoiceSign(e)
    }
}

impl From<lightning_invoice::ParseOrSemanticError> for Error {
    fn from(e: lightning_invoice::ParseOrSemanticError) -> Self {
        Error::LdkInvoiceParse(e)
    }
}

impl From<lightning::util::errors::APIError> for Error {
    fn from(e: lightning::util::errors::APIError) -> Self {
        Error::LdkApi(e)
    }
}

impl From<lightning::ln::msgs::LightningError> for Error {
    fn from(e: lightning::ln::msgs::LightningError) -> Self {
        Error::LdkMsg(e)
    }
}

impl From<database::Error> for Error {
    fn from(e: database::Error) -> Self {
        Error::Db(e)
    }
}

impl From<tindercrypt::errors::Error> for Error {
    fn from(e: tindercrypt::errors::Error) -> Self {
        Error::TinderCrypt(e)
    }
}

impl From<macaroon::MacaroonError> for Error {
    fn from(e: macaroon::MacaroonError) -> Self {
        Error::Macaroon(e)
    }
}

// TODO: since this is our library maybe we just want to map
// these to the underlying errors instead of being wrapped again
impl From<bdk_ldk::Error> for Error {
    fn from(e: bdk_ldk::Error) -> Error {
        Error::BdkLdk(e)
    }
}
