// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};

pub mod admin;
pub mod node;

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginationRequest {
    pub page: u32,
    pub take: u32,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub query: Option<String>,
}

impl Default for PaginationRequest {
    fn default() -> Self {
        Self {
            page: 0,
            take: 10,
            query: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PaymentsFilter {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub origin: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ForwardedPaymentsFilter {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub from_channel_id: Option<String>,
    pub to_channel_id: Option<String>,
    pub from_hours_since_epoch: Option<u64>,
    pub to_hours_since_epoch: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListPaymentsParams {
    pub page: u32,
    pub take: u32,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub origin: Option<String>,
    pub status: Option<String>,
    pub query: Option<String>,
}

impl Default for ListPaymentsParams {
    fn default() -> Self {
        Self {
            page: 1,
            take: 10,
            query: None,
            origin: None,
            status: None,
        }
    }
}

impl From<ListPaymentsParams> for PaymentsFilter {
    fn from(params: ListPaymentsParams) -> Self {
        Self {
            origin: params.origin,
            status: params.status,
        }
    }
}

impl From<ListPaymentsParams> for PaginationRequest {
    fn from(params: ListPaymentsParams) -> Self {
        Self {
            page: params.page,
            take: params.take,
            query: params.query,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListChannelsParams {
    pub page: u32,
    pub take: u32,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub query: Option<String>,
}

impl Default for ListChannelsParams {
    fn default() -> Self {
        Self {
            page: 1,
            take: 10,
            query: None,
        }
    }
}

impl From<ListChannelsParams> for PaginationRequest {
    fn from(params: ListChannelsParams) -> Self {
        Self {
            page: params.page,
            take: params.take,
            query: params.query,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListTransactionsParams {
    pub page: u32,
    pub take: u32,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub query: Option<String>,
}

impl Default for ListTransactionsParams {
    fn default() -> Self {
        Self {
            page: 1,
            take: 10,
            query: None,
        }
    }
}

impl From<ListTransactionsParams> for PaginationRequest {
    fn from(params: ListTransactionsParams) -> Self {
        Self {
            page: params.page,
            take: params.take,
            query: params.query,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListKnownPeersParams {
    pub page: u32,
    pub take: u32,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub query: Option<String>,
}

impl Default for ListKnownPeersParams {
    fn default() -> Self {
        Self {
            page: 0,
            take: 10,
            query: None,
        }
    }
}

impl From<ListKnownPeersParams> for PaginationRequest {
    fn from(params: ListKnownPeersParams) -> Self {
        Self {
            page: params.page,
            take: params.take,
            query: params.query,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListClusterNodesParams {
    pub page: u32,
    pub take: u32,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub query: Option<String>,
}

impl Default for ListClusterNodesParams {
    fn default() -> Self {
        Self {
            page: 0,
            take: 10,
            query: None,
        }
    }
}

impl From<ListClusterNodesParams> for PaginationRequest {
    fn from(params: ListClusterNodesParams) -> Self {
        Self {
            page: params.page,
            take: params.take,
            query: params.query,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginationResponse {
    pub has_more: bool,
    pub total: u64,
}
