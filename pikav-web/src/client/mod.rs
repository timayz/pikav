#[cfg(feature = "hydrate")]
mod hydrate;

#[cfg(not(feature = "hydrate"))]
mod ssr;

#[cfg(feature = "hydrate")]
pub use hydrate::*;

#[cfg(not(feature = "hydrate"))]
pub use ssr::*;
