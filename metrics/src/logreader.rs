use log::{debug, error, info, warn};
use std::os::unix::prelude::MetadataExt;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    sync::watch,
    time,
};
use std::io::Result;
use std::time::Duration;


use crate::{logprocessor, Telemetry};


pub async fn read_file(log_file: &str, metric_obj: &Telemetry, sleep_time: u64, mut termination_rx: watch::Receiver<()>) -> Result<()> {
    loop {
        tokio::select! {
            biased;
            _ = termination_rx.changed() => return Ok(()),

            // if metrics adapter starts when named pipe already exists and hasura overwrites it
            // after - original named pipe will be blocked waiting for input and never open.
            // this timeout will handle such situation by retrying long file opens
            _ = time::sleep(Duration::from_millis(1000)) => (),

            result = File::open(log_file) => {
                match result {
                    Ok(file) => {
                        info!("Hasura log file {} open, will follow the log", log_file);
                        match process_file(metric_obj, &file, sleep_time, termination_rx.clone()).await {
                            Ok(true) => (),
                            Ok(false) => return Ok(()),
                            Err(e) => {
                                warn!("Error reading logfile: {}", e);
                            }
                        };
                        info!("Need to reopen hasura log file {}", log_file);
                    }
                    Err(e) => {
                        error!("File {} could not be opened ({}). Will wait a little and then try again...", log_file, e);

                        tokio::select! {
                            biased;
                            _ = termination_rx.changed() => return Ok(()),
                            _ = time::sleep(Duration::from_millis(1000)) => (),
                        }
                    }
                }
            }
        }
    }
}

async fn process_file(metric_obj: &Telemetry, file: &File, sleep_time: u64, mut termination_rx: watch::Receiver<()>) -> Result<bool> {
    let reader = BufReader::new(file.try_clone().await?);
    let mut lines = reader.lines();

    loop {
        tokio::select! {
            biased;
            _ = termination_rx.changed() => return Ok(false),

            next_line = lines.next_line() => {
                if was_file_removed(&file).await? {
                    return Ok(true)
                }

                if let Some(line) = next_line? {
                    debug!("Reading line from logfile");
                    logprocessor::log_processor(&line, metric_obj).await;
                } else {
                    time::sleep(Duration::from_millis(sleep_time)).await;
                }
            }
        }
    }
}

async fn was_file_removed(file: &File) -> Result<bool> {
    Ok(file.metadata().await?.nlink() == 0)
}
