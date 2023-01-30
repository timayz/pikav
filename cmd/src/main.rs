mod serve;
mod publish;

use clap::{arg, Command};
use serve::Serve;
use publish::Publish;

fn cli() -> Command<'static> {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("serve")
                .about("Run the pikav server")
                .arg(arg!(-c --config <CONFIG>).required(false)),
        )
        .subcommand(
            Command::new("publish")
                .about("Publish event to pikav server")
                .arg(arg!(-c --config <CONFIG>).required(false)),
        )
}

#[actix_rt::main]
async fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("serve", sub_matches)) => {
            let s = match Serve::new(sub_matches.value_of("config").unwrap_or_default()) {
                Ok(s) => s,
                Err(e) => panic!("{e}"),
            };

            if let Err(e) = s.run().await {
                panic!("{e}");
            }
        }
        Some(("publish", sub_matches)) => {
            let p = match Publish::new(sub_matches.value_of("config").unwrap_or_default()) {
                Ok(s) => s,
                Err(e) => panic!("{e}"),
            };

            if let Err(e) = p.run().await {
                panic!("{e}");
            }
        }
        _ => unreachable!(),
    }
}
