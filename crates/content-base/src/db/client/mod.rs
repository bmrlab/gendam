#[cfg(feature = "embedded-db")]
pub mod embedded;

#[cfg(not(feature = "embedded-db"))]
pub mod remote;
