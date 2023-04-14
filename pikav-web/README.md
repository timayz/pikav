A library that help you send event to client with topic subscription

---

## Getting Started

```rust
use serde_json::json;
use pikav_client::{Client, ClientOptions, Event};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pikav = Pikav::new();

    let filter = TopicFilter::new("todos/+")?;

    pikav.subscribe(SubscribeOptions {
            filter,
            user_id: "xx-user-id-xx",
            client_id: "xx-client-id-xx",
        })
        .ok();
}
```
