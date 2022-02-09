// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};

pub struct MacaroonHeader(pub Option<http::HeaderValue>);

#[async_trait]
impl<B> FromRequest<B> for MacaroonHeader
where
    B: Send,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let headers = req.headers().ok_or((
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "headers already extracted",
        ))?;

        if let Some(value) = headers.get("macaroon") {
            Ok(Self(Some(value.clone())))
        } else {
            Ok(Self(None))
        }
    }
}
