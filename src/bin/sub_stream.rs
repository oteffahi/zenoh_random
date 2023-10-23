use clap::{App, Arg};
use futures::prelude::*;
use futures::select;
use std::convert::TryFrom;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;
// use async_std::task::sleep;
// use std::time::Duration;

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let config = parse_args();
    let sub_key_expr = KeyExpr::try_from("test/random").unwrap().into_owned();
    let quer_key_expr_str = "test/average";
    let quer_key_expr = KeyExpr::try_from(quer_key_expr_str).unwrap().into_owned();

    let mut sum: i64 = 0; // i64 to avoid overflow
    let mut nb_values: u128 = 0; // u128 for max scalability

    println!("Opening session...");
    let session = zenoh::open(config).res().await.unwrap();

    println!("Declaring Subscriber on '{}'...", &sub_key_expr);

    let subscriber = session
        .declare_subscriber(&sub_key_expr)
        .res()
        .await
        .unwrap();
    let queryable = session
        .declare_queryable(&quer_key_expr)
        .res()
        .await
        .unwrap();

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
                    Ok(v) => {
                        sum += v as i64;
                        nb_values += 1;
                    },
                    Err(e) => println!("Error occurred: {e}"),
                }
            },
            query = queryable.recv_async() => {
                if let Ok(query) = query {
                    let mut current_average: f64 = 0.0;
                    // sleep(Duration::from_millis(5000)).await; // simulate work before calculating average
                    if nb_values > 0 {
                        current_average = sum as f64 / nb_values as f64
                    }
                    println!(">> [Queryable] Received query for '{}': Responding with current average {}", query.key_expr(), current_average);
                    // sleep(Duration::from_millis(5000)).await; // simulate work after calculating average
                    query.reply(Ok(Sample::try_from(quer_key_expr_str, current_average).unwrap()))
                    .res()
                    .await
                    .unwrap();
                }
            }

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
