use futures::Future;
use leptos::Scope;
use pikav::Event;
use serde_json::Value;

use leptos::{on_cleanup, provide_context, use_context};

use crate::Client;

pub fn pikav_context(cx: Scope, client: Client) {
    pikav_context_with_clients(cx, vec![client]);
}

pub fn pikav_context_with_clients(cx: Scope, clients: Vec<Client>) {
    provide_context(cx, clients.clone());

    on_cleanup(cx, move || clients.iter().for_each(|c| c.close()));
}

pub fn use_client(cx: Scope) -> Client {
    use_client_with(cx, 0)
}

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