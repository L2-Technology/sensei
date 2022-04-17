use tonic::{metadata::MetadataMap, Status};

pub fn raw_macaroon_from_metadata(metadata: MetadataMap) -> Result<String, tonic::Status> {
    let macaroon = metadata.get("macaroon");

    if macaroon.is_none() {
        return Err(Status::unauthenticated("macaroon is required"));
    }

    macaroon
        .unwrap()
        .to_str()
        .map(String::from)
        .map_err(|_e| Status::unauthenticated("invalid macaroon: must be ascii"))
}
