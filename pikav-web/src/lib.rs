mod client;
#[cfg(feature = "leptos")]
pub mod leptos;

pub use client::Client;
pub use gloo_net::http::Headers;
