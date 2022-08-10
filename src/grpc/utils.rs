use tonic::{metadata::MetadataMap, Status};

pub fn raw_macaroon_from_metadata(metadata: MetadataMap) -> Result<Option<String>, Status> {
    let macaroon_opt = metadata.get("macaroon");

    match macaroon_opt {
        Some(macaroon) => macaroon
            .to_str()
            .map(|s| Some(String::from(s)))
            .map_err(|_e| Status::unauthenticated("invalid macaroon: must be ascii")),
        None => Ok(None),
    }
}

pub fn raw_token_from_metadata(metadata: MetadataMap) -> Result<Option<String>, Status> {
    let token_opt = metadata.get("token");

    match token_opt {
        Some(token) => token
            .to_str()
            .map(|s| Some(String::from(s)))
            .map_err(|_e| Status::unauthenticated("invalid token: must be ascii")),
        None => Ok(None),
    }
}
