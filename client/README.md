A client that help you publish to pikav server

---

## Getting Started

```rust
use serde_json::json;
use pikav_client::{Client, ClientOptions, Event};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pikva_client = Client::new(ClientOptions {
        url: "http://127.0.0.1:6750".to_owned(),
        namespace: None,
    });

    client.publish(vec![Event::new(
            user.0,
            "todos/1",
            "Deleted",
            json!({
                "id": id.to_owned()
            }),
        )
        .unwrap()]);
}
```
