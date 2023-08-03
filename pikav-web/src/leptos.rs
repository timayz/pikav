use cfg_if::cfg_if;
use futures::Future;
use pikav::Event;
use serde_json::Value;

#[cfg(feature = "leptos-hydrate")]
use leptos::{on_cleanup, provide_context, use_context};

use crate::Client;

pub fn pikav_context(client: Client) {
    pikav_context_with_clients(vec![client]);
}

cfg_if! {
    if #[cfg(feature = "leptos-hydrate")] {
        pub fn pikav_context_with_clients(clients: Vec<Client>) {
            provide_context(clients.clone());

            on_cleanup(move || clients.iter().for_each(|c| c.close()));
        }
    } else {
        pub fn pikav_context_with_clients(_clients: Vec<Client>) {}
    }
}

pub fn use_client() -> Client {
    use_client_with(0)
}

cfg_if! {
    if #[cfg(feature = "leptos-hydrate")] {
        pub fn use_client_with(id: usize) -> Client {
            use_context::<Vec<Client>>()
                .expect("Pikav provider not configured correctly")
                .get(id)
                .unwrap_or_else(|| panic!("to get client {id}"))
                .clone()
        }
    } else {
        pub fn use_client_with(_id: usize) -> Client {
            Client::new("")
        }
    }
}

pub fn use_subscribe<Fut>(
    filter: impl Into<String> + 'static,
    listener: impl Fn(Event<Value, Value>) -> Fut + 'static,
) where
    Fut: Future<Output = ()> + 'static + Send,
{
    use_subscribe_with(0, filter, listener);
}

cfg_if! {
    if #[cfg(feature = "leptos-hydrate")] {
        pub fn use_subscribe_with<Fut>(
            id: usize,
            filter: impl Into<String> + 'static,
            listener: impl Fn(Event<Value, Value>) -> Fut + 'static,
        ) where
            Fut: Future<Output = ()> + 'static + Send,
        {
            let unsubscribe = use_client_with(id).subscribe(filter, listener);

            on_cleanup(unsubscribe);
        }
    } else {
        pub fn use_subscribe_with<Fut>(
            _id: usize,
            _filter: impl Into<String> + 'static,
            _listener: impl Fn(Event<Value, Value>) -> Fut + 'static,
        ) where
            Fut: Future<Output = ()> + 'static + Send,
        {
        }
    }
}
