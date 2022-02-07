pub mod admin;
pub mod node;

#[derive(Debug)]
pub enum Error {
    Generic(String),
}
