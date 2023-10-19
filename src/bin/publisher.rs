use async_std::task::sleep;
use clap::{App, Arg};
use std::time::Duration;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;
use rand::Rng;

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let config = parse_args();
    let key_expr = String::from("test/random");

    println!("Opening session...");
    let session = zenoh::open(config).res().await.unwrap();

    println!("Declaring Publisher on '{key_expr}'...");
    let publisher = session.declare_publisher(&key_expr).res().await.unwrap();

    let mut rng = rand::thread_rng();
    loop {
        // sleep 2 seconds
        sleep(Duration::from_secs(2)).await;
        // generate random i32 value
        let buf:i32 = rng.gen();
        // log
        println!("Putting Data ('{}': '{}')...", &key_expr, buf);
        // publish the value
        publisher.put(buf).res().await.unwrap();
    }
}

fn parse_args() -> Config {
    let args = App::new("Zenoh Pub Random Number")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode (peer by default).")
                .possible_values(["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[ENDPOINT]...  'Endpoints to connect to.'",
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