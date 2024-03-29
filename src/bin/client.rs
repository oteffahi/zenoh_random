use clap::{App, Arg};
use std::convert::TryFrom;
use std::time::Duration;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;
mod utils;
use utils::QueryableData;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let (config, target, timeout) = parse_args();
    let selector = "test/average";

    println!("Opening session...");
    let session = zenoh::open(config).res().await.unwrap();

    println!("Sending Query '{selector}'...");
    let replies = session
        .get(selector)
        .consolidation(ConsolidationMode::None) // no consolidation, to receive all replies and compare counters
        .target(target)
        .timeout(timeout)
        .res()
        .await
        .unwrap();

    let mut max: u128 = 0;
    let mut average = 0f64;
    while let Ok(reply) = replies.recv_async().await {
        match reply.sample {
            Ok(sample) => {
                let resp = String::try_from(sample.value).unwrap();
                let resp: QueryableData = serde_json::from_str(&resp).unwrap();
                println!(
                    ">> Received ('{}': '{} {}')",
                    sample.key_expr.as_str(),
                    resp.average,
                    resp.nb_values
                );
                if resp.nb_values > max {
                    max = resp.nb_values;
                    average = resp.average;
                }
            }
            Err(err) => println!(">> Received (ERROR: '{}')", String::try_from(&err).unwrap()),
        }
    }
    println!("Selected average: {average} with nb_values={max}");
}

fn parse_args() -> (Config, QueryTarget, Duration) {
    let args = App::new("Zenoh Client Random Number")
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
        .arg(
            Arg::from_usage("-t, --target=[TARGET] 'The target queryables of the query'")
                .possible_values(["BEST_MATCHING", "ALL", "ALL_COMPLETE"])
                .default_value("BEST_MATCHING"),
        )
        .arg(
            Arg::from_usage("-o, --timeout=[TIME] 'The query timeout in milliseconds'")
                .default_value("10000"),
        )
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

    let target = match args.value_of("target") {
        Some("BEST_MATCHING") => QueryTarget::BestMatching,
        Some("ALL") => QueryTarget::All,
        Some("ALL_COMPLETE") => QueryTarget::AllComplete,
        _ => QueryTarget::default(),
    };

    let timeout = Duration::from_millis(args.value_of("timeout").unwrap().parse::<u64>().unwrap());

    (config, target, timeout)
}
