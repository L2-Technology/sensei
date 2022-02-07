pub mod adaptor;
pub mod admin;
pub mod node;

pub mod sensei {
    tonic::include_proto!("sensei");
}
