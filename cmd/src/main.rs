mod publish;
mod serve;

use clap::{arg, Command};
use publish::Publish;
use serve::Serve;

fn cli() -> Command {
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
            let s = match Serve::new(
                sub_matches
                    .get_one::<String>("config")
                    .unwrap_or(&"".to_owned()),
            ) {
                Ok(s) => s,
                Err(e) => panic!("{e}"),
            };

            if let Err(e) = s.run().await {
                panic!("{e}");
            }
        }
        Some(("publish", sub_matches)) => {
            let p = match Publish::new(
                sub_matches
                    .get_one::<String>("config")
                    .unwrap_or(&"".to_owned()),
            ) {
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
