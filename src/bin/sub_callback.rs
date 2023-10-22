use clap::{App, Arg};
use futures::prelude::*;
use futures::select;
use std::convert::TryFrom;
use std::sync::Arc;
use std::sync::RwLock;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;
use zenoh::prelude::sync::SyncResolve;

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let config = parse_args();
    let sub_key_expr = KeyExpr::try_from("test/random").unwrap().into_owned();
    let quer_key_expr_str = "test/average";
    let quer_key_expr = KeyExpr::try_from(quer_key_expr_str).unwrap().into_owned();

    let sum: Arc<RwLock<i64>> = Arc::new(RwLock::new(0)); // i64 to avoid overflow
    let nb_values: Arc<RwLock<u128>> = Arc::new(RwLock::new(0)); // u128 for max scalability
    // clone to access sum and nb_values after move in closure
    let sum_clone = sum.clone();
    let nb_values_clone = nb_values.clone();

    println!("Opening session...");
    let session = zenoh::open(config).res_async().await.unwrap();

    println!("Declaring Subscriber on '{}'...", &sub_key_expr);

    let _subscriber = session
        .declare_subscriber(&sub_key_expr)
        .callback(move |sample| {
            println!(
                ">> [Subscriber] Received {} ('{}': '{}')",
                sample.kind,
                sample.key_expr.as_str(),
                sample.value
            );
            let value = i32::try_from(sample.value);
            match value {
                Ok(v) => {
                    let mut sum = sum.write().unwrap();
                    let mut nb_values = nb_values.write().unwrap();
                    *sum += v as i64;
                    *nb_values += 1;
                }
                Err(e) => println!("Error occurred: {e}"),
            }
        })
        .res_async()
        .await
        .unwrap();

    let _queryable = session
        .declare_queryable(&quer_key_expr)
        .callback(move |query| {
            let mut current_average: f64 = 0.0;
            let sum = *sum_clone.read().unwrap();
            let nb_values = *nb_values_clone.read().unwrap();

            if nb_values > 0 {
                current_average = sum as f64 / nb_values as f64;
            }
            println!(
                ">> [Queryable] Received query for '{}': Responding with current average {}",
                query.key_expr(),
                current_average
            );
            query
                .reply(Ok(
                    Sample::try_from(quer_key_expr_str, current_average).unwrap()
                ))
                .res_sync()
                .unwrap();
        })
        .res_sync()
        .unwrap();

    println!("Enter 'q' to quit...");
    let mut stdin = async_std::io::stdin();
    let mut input = [0_u8];
    loop {
        select!(
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
