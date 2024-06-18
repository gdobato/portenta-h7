#[cfg(feature = "async")]
pub mod async_impl;

#[cfg(not(feature = "async"))]
pub mod non_async_impl;

pub use fugit::HertzU32;
pub const CORE_FREQUENCY: HertzU32 = HertzU32::from_raw(480_000_000);
