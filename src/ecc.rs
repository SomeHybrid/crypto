pub(crate) mod field;
pub mod x25519;

#[derive(Debug)]
pub struct InvalidKey;

impl std::fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This key is an invalid size!")
    }
}

impl std::error::Error for InvalidKey {}
