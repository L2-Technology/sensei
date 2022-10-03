// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::{
    fmt::{self, Display},
    io::ErrorKind,
};

#[derive(Debug)]
pub enum Error {
    Generic(String),
    Db(migration::DbErr),
    TinderCrypt(tindercrypt::errors::Error),
    Macaroon(macaroon::MacaroonError),
    Io(std::io::Error),
    Secp256k1(bitcoin::secp256k1::Error),
    Bdk(bdk::Error),
    BitcoinRpc(bitcoincore_rpc::Error),
    LdkApi(lightning::util::errors::APIError),
    LdkMsg(lightning::ln::msgs::LightningError),
    LdkInvoice(lightning_invoice::payment::PaymentError),
    LdkInvoiceSign(lightning_invoice::SignOrCreationError),
    LdkInvoiceParse(lightning_invoice::ParseOrSemanticError),
    InvalidSeedLength,
    InvalidEntropyLength,
    FailedToWriteEntropy,
    EntropyNotFound,
    MacaroonNotFound,
    Unauthenticated,
    InvalidMacaroon,
    AdminNodeNotStarted,
    AdminNodeNotCreated,
    FundingGenerationNeverHappened,
    ChannelOpenRejected(String),
    NodeBeingStartedAlready,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Error::Generic(e) => e.to_string(),
            Error::Db(e) => e.to_string(),
            Error::Macaroon(_e) => "macaroon error".to_string(),
            Error::TinderCrypt(e) => e.to_string(),
            Error::Io(e) => e.to_string(),
            Error::Secp256k1(e) => e.to_string(),
            Error::Bdk(e) => e.to_string(),
            Error::BitcoinRpc(e) => e.to_string(),
            Error::LdkApi(e) => format!("{:?}", e),
            Error::LdkMsg(e) => format!("{:?}", e),
            Error::LdkInvoice(e) => format!("{:?}", e),
            Error::LdkInvoiceSign(e) => e.to_string(),
            Error::LdkInvoiceParse(e) => e.to_string(),
            Error::InvalidSeedLength => String::from("invalid seed length"),
            Error::EntropyNotFound => String::from("entropy not found for node"),
            Error::MacaroonNotFound => String::from("macaroon not found for node"),
            Error::FailedToWriteEntropy => String::from("failed to write entropy"),
            Error::Unauthenticated => String::from("unauthenticated"),
            Error::InvalidMacaroon => String::from("invalid macaroon"),
            Error::AdminNodeNotCreated => String::from("admin node not created"),
            Error::AdminNodeNotStarted => String::from("admin node not started"),
            Error::NodeBeingStartedAlready => String::from("node already being started"),
            Error::FundingGenerationNeverHappened => {
                String::from("funding generation for request never happened")
            }
            Error::ChannelOpenRejected(reason) => {
                format!("Channel open rejected by peer: {:?}", reason)
            }
            Error::InvalidEntropyLength => String::from("invalid entropy length"),
        };
        write!(f, "{}", str)
    }
}

impl From<migration::DbErr> for Error {
    fn from(e: migration::DbErr) -> Error {
        Error::Db(e)
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

impl From<bitcoincore_rpc::Error> for Error {
    fn from(e: bitcoincore_rpc::Error) -> Error {
        Error::BitcoinRpc(e)
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

impl From<Error> for std::io::Error {
    fn from(e: Error) -> std::io::Error {
        std::io::Error::new(ErrorKind::Other, e.to_string())
    }
}
