use axum::response::{IntoResponse, Response};
use headers::HeaderValue;
use http::StatusCode;
use tower_cookies::Cookies;

pub fn get_macaroon_hex_str_from_cookies_or_header(
    cookies: &Cookies,
    macaroon: Option<HeaderValue>,
) -> Result<String, Response> {
    match macaroon {
        Some(macaroon) => {
            let res = macaroon
                .to_str()
                .map(|str| str.to_string())
                .map_err(|_| (StatusCode::UNAUTHORIZED, "unauthorized").into_response());
            res
        }
        None => match cookies.get("macaroon") {
            Some(macaroon_cookie) => {
                let macaroon_cookie_str = macaroon_cookie.value().to_string();
                Ok(macaroon_cookie_str)
            }
            None => Err((StatusCode::UNAUTHORIZED, "unauthorized").into_response()),
        },
    }
}
