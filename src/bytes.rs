use std::convert::AsRef;

pub trait FromBytes: Sized {
    type Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Self::Error>;
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}
