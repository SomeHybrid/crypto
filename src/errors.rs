use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub struct InvalidMac;

impl Eq for InvalidMac {}

impl fmt::Display for InvalidMac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid MAC detected. This message may be tampered with."
        )
    }
}

impl fmt::Debug for InvalidMac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid MAC detected. This message may be tampered with."
        )
    }
}

impl Error for InvalidMac {}
