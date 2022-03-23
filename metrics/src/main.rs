use actix_web::{get, App, HttpServer, Responder};
use clap::Arg;
use log::{error, info, warn};
use prometheus::{Encoder, TextEncoder};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

mod logprocessor;

async fn read_fille(cfg: &Configuration) -> std::io::Result<()> {
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

struct Configuration {
    listen_addr: String,
    log_file: String,
    sleep_time: u64,
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
                    .takes_value(true),
            )
            .arg(
                Arg::new("sleep")
                    .long("sleep")
                    .env("SLEEP_TIME")
                    .default_value("5000")
                    .takes_value(true),
            )
            .get_matches();

        Configuration {
            listen_addr: matches.value_of("listen").expect("required").to_string(),
            log_file: matches.value_of("logfile").expect("required").to_string(),
            sleep_time: matches
                .value_of_t("sleep")
                .expect("can't configure sleep time"),
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Configuration::default();

    let res = tokio::try_join!(webserver(&config), read_fille(&config));

    match res {
        Err(e) => {
            error!("System error: {}", e);
        }
        _ => {
            info!("bye bye");
        }
    };
}
