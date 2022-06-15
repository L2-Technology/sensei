use tonic::{metadata::MetadataMap, Status};

pub fn raw_macaroon_from_metadata(metadata: MetadataMap) -> Result<String, Status> {
    let macaroon_opt = metadata.get("macaroon");

    match macaroon_opt {
        Some(macaroon) => macaroon
            .to_str()
            .map(String::from)
            .map_err(|_e| Status::unauthenticated("invalid macaroon: must be ascii")),
        None => Err(Status::unauthenticated("macaroon is required")),
    }
}
