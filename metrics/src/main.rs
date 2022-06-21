use std::sync::mpsc;

use actix_web::{App, get, HttpServer, Responder};
use clap::Parser;

use lazy_static::lazy_static;
use log::{info, warn, debug};
use prometheus::{Encoder, TextEncoder};
use prometheus::{IntCounterVec, register_int_counter_vec};

mod logreader;
mod logprocessor;
mod collectors;

lazy_static! {
    pub(crate) static ref ERRORS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "hasura_errors_total",
        "the total number of errors per collector",
        &["collector"]
    )
    .unwrap();
}



#[get("/metrics")]
async fn metrics() -> impl Responder {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();

    // Gather the metrics.
    let metric_families = prometheus::gather();
    // Encode them to send.
    encoder.encode(&metric_families, &mut buffer).unwrap();

    String::from_utf8(buffer.clone()).unwrap()
}

async fn webserver(cfg: &Configuration) -> std::io::Result<()> {
    warn!("Starting metric server @ {}", cfg.listen_addr);
    HttpServer::new(|| App::new().service(metrics))
        .bind(&cfg.listen_addr)?
        .run()
        .await
}

#[derive(clap::ValueEnum, Clone,Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Collectors {
    CronTriggers,
    EventTriggers,
    ScheduledEvents,
    MetadataInconsistency,
}

#[derive(Parser,Debug)]
#[clap(author, version, about)]
pub(crate) struct Configuration {
    #[clap(name ="listen", long = "listen", env = "LISTEN_ADDR", default_value = "0.0.0.0:9090")]
    listen_addr: String,

    #[clap(name ="hasura-endpoint", long = "hasura-endpoint", env = "HASURA_GRAPHQL_ENDPOINT", default_value = "http://localhost:8080")]
    hasura_addr: String,

    #[clap(name ="hasura-admin-secret", long = "hasura-admin-secret", env = "HASURA_GRAPHQL_ADMIN_SECRET")]
    hasura_admin: Option<String>,

    #[clap(name ="logfile", long = "logfile", env = "LOG_FILE")]
    log_file: String,

    #[clap(name ="sleep", long = "sleep", env = "SLEEP_TIME", default_value = "1000")]
    sleep_time: u64,

    #[clap(name ="collect-interval", long = "collect-interval", env = "COLLECT_INTERVAL", default_value = "15000")]
    collect_interval: u64,

    #[clap(name ="exclude_collectors", long = "exclude_collectors", env = "EXCLUDE_COLLECTORS", value_parser, value_delimiter(';'))]
    disabled_collectors: Vec<Collectors>,
}

async fn signal_handler_ctrl_c(tx: mpsc::Sender<()>) -> std::io::Result<()> {
    tokio::signal::ctrl_c().await?;
    warn!("Terminating due to ctrl+c");
    let _ = tx.send(());
    Ok(())
}

fn signal_handler() -> mpsc::Receiver<()> {
    let (terminate_tx, terminate_rx) = mpsc::channel();
    tokio::spawn(signal_handler_ctrl_c(terminate_tx));
    terminate_rx
}


#[tokio::main]
async fn main() {
    env_logger::init();
    let mut config = Configuration::parse();

    if config.hasura_admin.is_none() {
        let admin_collectors = [
            Collectors::CronTriggers,
            Collectors::EventTriggers,
            Collectors::ScheduledEvents,
            Collectors::MetadataInconsistency,
        ];

        config.disabled_collectors.extend_from_slice(&admin_collectors);

        warn!("No Hasura admin secret provided, disabling following collectors: {:?}", &admin_collectors);
    }

    config.disabled_collectors.sort();
    config.disabled_collectors.dedup();

    println!("hasura-metrics-adapter on {0} for hasura at {1} parsing hasura log '{2}'", config.listen_addr, config.hasura_addr, config.log_file);

    debug!("Configuration: {:?}", config);

    let terminate_rx = signal_handler();

    let res = tokio::try_join!(
        webserver(&config),
        logreader::read_file(&config.log_file, config.sleep_time, &terminate_rx),
        collectors::run_metadata_collector(&config, &terminate_rx)
    );
    match res {
        Err(e) => {
            panic!("System error: {}", e);
        }
        _ => {
            info!("bye bye");
        }
    };
}
