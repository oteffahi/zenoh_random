use clap::{App, Arg};
use futures::prelude::*;
use futures::select;
use std::convert::TryFrom;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let config = parse_args();
    let key_expr = KeyExpr::try_from("test/random")
    .unwrap()
    .into_owned();

    println!("Opening session...");
    let session = zenoh::open(config).res().await.unwrap();

    println!("Declaring Subscriber on '{}'...", &key_expr);

    let subscriber = session.declare_subscriber(&key_expr).res().await.unwrap();

    println!("Enter 'q' to quit...");
    let mut stdin = async_std::io::stdin();
    let mut input = [0_u8];
    loop {
        select!(
            sample = subscriber.recv_async() => {
                let sample = sample.unwrap();
                println!(">> [Subscriber] Received {} ('{}': '{}')",
                    sample.kind, sample.key_expr.as_str(), sample.value);
                let value = i32::try_from(sample.value);
                match value {
                    Ok(v) => println!("Got random value {v}"),
                    Err(e) => println!("Error occured: {e}"),
                }
            },

            _ = stdin.read_exact(&mut input).fuse() => {
                match input[0] {
                    b'q' => break,
                    _ => (),
                }
            }
        );
    }
}

fn parse_args() -> Config {
    let args = App::new("Zenoh Sub Random Number")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The zenoh session mode (peer by default).")
                .possible_values(["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[ENDPOINT]...   'Endpoints to connect to.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listen=[ENDPOINT]...   'Endpoints to listen on.'",
        ))
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Config::from_file(conf_file).unwrap()
    } else {
        Config::default()
    };
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(values) = args.values_of("connect") {
        config.connect.endpoints = values.map(|v| v.parse().unwrap()).collect();
    }
    if let Some(values) = args.values_of("listen") {
        config.listen.endpoints = values.map(|v| v.parse().unwrap()).collect();
    }
    if args.is_present("no-multicast-scouting") {
        config.scouting.multicast.set_enabled(Some(false)).unwrap();
    }

    config
}