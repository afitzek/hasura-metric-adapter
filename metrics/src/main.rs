use tokio::io::{stdin, BufReader, AsyncBufReadExt};
use actix_web::{get, App, HttpServer, Responder};
use prometheus::{TextEncoder, Encoder};
use clap::{Arg, Command};

mod logprocessor;

async fn parsestdin() -> std::io::Result<()> {
    let stdin = stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        logprocessor::log_processor(&line).await;
    }
    Ok(())
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
    println!("Starting metric server @ {}", cfg.listen_addr);
    HttpServer::new(|| App::new().service(metrics))
        .bind(&cfg.listen_addr)?
        .run()
        .await
}

struct Configuration {
    listen_addr: String
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
                .takes_value(true)
            )
            .get_matches();

        Configuration {
            listen_addr: matches.value_of("listen").expect("required").to_string()
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Configuration::default();

    let res = tokio::try_join!(webserver(&config),parsestdin());

    match res {
        Err(e) => {
            eprintln!("System error: {}", e);
        },
        _ => {
            println!("bye bye");
        }
    };
}
