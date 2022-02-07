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
