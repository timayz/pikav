use futures::Future;
use leptos::Scope;
use pikav::Event;
use serde_json::Value;

use crate::Client;

pub fn pikav_context(cx: Scope, client: Client) {
    pikav_context_with_clients(cx, vec![client]);
}

pub fn pikav_context_with_clients(_cx: Scope, _clients: Vec<Client>) {}

pub fn use_client(cx: Scope) -> Client {
    use_client_with(cx, 0)
}

pub fn use_client_with(_cx: Scope, _id: usize) -> Client {
    Client::new("")
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

pub fn use_subscribe_with<Fut>(
    _cx: Scope,
    _id: usize,
    _filter: impl Into<String> + 'static,
    _listener: impl Fn(Event<Value, Value>) -> Fut + 'static,
) where
    Fut: Future<Output = ()> + 'static + Send,
{
}
