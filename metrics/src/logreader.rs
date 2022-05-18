use std::sync::mpsc;
use log::{error, info};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use std::io::Result;

use crate::{logprocessor, util};


pub async fn read_file(log_file: &String, sleep_time: u64, rx: &mpsc::Receiver<()>) -> Result<()> {
    loop {
        if util::should_stop(rx) {
            return Ok(());
        }
        match File::open(log_file).await {
            Ok(file) => {
                info!("Hasura log file {} open, will follow the log", log_file);
                process_file(file, sleep_time, rx).await?;
            }
            Err(e) => {
                error!("File {} could not be opened ({}). Will wait a little and then try again...", log_file, e);
                tokio::time::sleep(std::time::Duration::from_millis(sleep_time)).await;
            }
        }
    }
}

async fn process_file(file: File, sleep_time: u64, rx: &mpsc::Receiver<()>) -> Result<()> {
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    loop {
        if util::should_stop(rx) {
            return Ok(());
        }

        if let Some(line) = lines.next_line().await? {
            logprocessor::log_processor(&line).await;
        } else {
            // check for file changes every sleep time ms.
            // This can be quite high, because usually one has a sample rate
            // for scraping the prometheus metrics of a couple of seconds
            tokio::time::sleep(std::time::Duration::from_millis(sleep_time)).await;
        }
    }
}