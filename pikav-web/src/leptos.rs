use futures::Future;
use leptos::Scope;
use pikav::Event;
use serde_json::Value;

#[cfg(feature = "hydrate")]
use leptos::{on_cleanup, provide_context, use_context};

use crate::Client;

pub fn pikav_context(cx: Scope, client: Client) {
    pikav_context_with_clients(cx, vec![client]);
}

#[cfg(not(feature = "hydrate"))]
pub fn pikav_context_with_clients(_cx: Scope, _clients: Vec<Client>) {}

#[cfg(feature = "hydrate")]
pub fn pikav_context_with_clients(cx: Scope, clients: Vec<Client>) {
    provide_context(cx, clients.clone());

    on_cleanup(cx, move || clients.iter().for_each(|c| c.close()));
}

pub fn use_client(cx: Scope) -> Client {
    use_client_with(cx, 0)
}

#[cfg(not(feature = "hydrate"))]
pub fn use_client_with(_cx: Scope, _id: usize) -> Client {
    Client::new("")
}

#[cfg(feature = "hydrate")]
pub fn use_client_with(cx: Scope, id: usize) -> Client {
    use_context::<Vec<Client>>(cx)
        .expect("Pikav provider not configured correctly")
        .get(id)
        .expect(&format!("to get client {id}"))
        .clone()
}

pub fn use_subscribe<Fut>(
    cx: Scope,
    filter: impl Into<String> + 'static,
    listener: impl Fn(Event<Value, Value>) -> Fut + 'static,
) where
    Fut: Future<Output = ()> + 'static + Send,
{
    use_subscribe_with(cx, 0, filter, listener);
}

#[cfg(not(feature = "hydrate"))]
pub fn use_subscribe_with<Fut>(
    _cx: Scope,
    _id: usize,
    _filter: impl Into<String> + 'static,
    _listener: impl Fn(Event<Value, Value>) -> Fut + 'static,
) where
    Fut: Future<Output = ()> + 'static + Send,
{
}

#[cfg(feature = "hydrate")]
pub fn use_subscribe_with<Fut>(
    cx: Scope,
    id: usize,
    filter: impl Into<String> + 'static,
    listener: impl Fn(Event<Value, Value>) -> Fut + 'static,
) where
    Fut: Future<Output = ()> + 'static + Send,
{
    let unsubscribe = use_client_with(cx, id).subscribe(filter, listener);

    on_cleanup(cx, move || unsubscribe());
}
