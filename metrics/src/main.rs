use actix_web::{get, App, HttpServer, Responder};
use clap::Arg;
use log::{error, info, warn};
use prometheus::{Encoder, TextEncoder};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use lazy_static::lazy_static;
use prometheus::{register_int_counter_vec, IntCounterVec};

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

async fn read_file(cfg: &Configuration) -> std::io::Result<()> {
    let input = File::open(&cfg.log_file).await?;
    let reader = BufReader::new(input);
    let mut lines = reader.lines();

    loop {
        if let Some(line) = lines.next_line().await? {
            logprocessor::log_processor(&line).await;
        } else {
            // check for file changes every sleep time ms.
            // This can be quite high, because usually one has a sample rate
            // for scraping the prometheus metrics of a couple of seconds
            tokio::time::sleep(std::time::Duration::from_millis(cfg.sleep_time)).await;
        }
    }
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

pub(crate) struct Configuration {
    listen_addr: String,
    hasura_addr: String,
    hasura_admin: String,
    log_file: String,
    sleep_time: u64,
    collect_interval: u64,
}

impl Default for Configuration {
    fn default() -> Self {
        let matches = clap::command!()
            .version(clap::crate_version!())
            .author(clap::crate_authors!("\n"))
            .about(clap::crate_description!())
            .arg(
                Arg::new("listen")
                    .long("listen")
                    .env("LISTEN_ADDR")
                    .default_value("0.0.0.0:9090")
                    .takes_value(true),
            )
            .arg(
                Arg::new("logfile")
                    .long("logfile")
                    .env("LOG_FILE")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::new("sleep")
                    .long("sleep")
                    .env("SLEEP_TIME")
                    .default_value("5000")
                    .takes_value(true),
            )
            .arg(
                Arg::new("collect-intervall")
                    .long("collect-intervall")
                    .env("COLLECT_INTERVALL")
                    .default_value("15000")
                    .takes_value(true),
            )
            .arg(
                Arg::new("hasura-endpoint")
                    .long("hasura-endpoint")
                    .env("HASURA_GRAPHQL_ENDPOINT")
                    .default_value("http://localhost:8080")
                    .takes_value(true),
            )
            .arg(
                Arg::new("hasura-admin-secret")
                    .long("hasura-admin-secret")
                    .env("HASURA_GRAPHQL_ADMIN_SECRET")
                    .required(true)
                    .takes_value(true),
            )
            .get_matches();

        Configuration {
            listen_addr: matches.value_of("listen").expect("required").to_string(),
            log_file: matches.value_of("logfile").expect("required").to_string(),
            hasura_addr: matches.value_of("hasura-endpoint").expect("required").trim_end().trim_end_matches("/").to_string(),
            hasura_admin: matches.value_of("hasura-admin-secret").expect("required").to_string(),
            sleep_time: matches
                .value_of_t("sleep")
                .expect("can't configure sleep time"),
            collect_interval: matches
                .value_of_t("collect-intervall")
                .expect("can't configure collect-intervall time"),
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Configuration::default();

    let res = tokio::try_join!(
        webserver(&config),
        read_file(&config),
        collectors::run_metadata_collector(&config)
    );

    match res {
        Err(e) => {
            error!("System error: {}", e);
        }
        _ => {
            info!("bye bye");
        }
    };
}
