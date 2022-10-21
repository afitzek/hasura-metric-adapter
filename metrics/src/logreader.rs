use std::sync::mpsc;
use std::sync::mpsc::{RecvTimeoutError,TryRecvError};
use log::{error, info, warn};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use std::io::Result;
use std::time::Duration;
use notify::{Watcher, watcher, RecursiveMode, DebouncedEvent};


use crate::{logprocessor, Telemetry};


pub async fn read_file(log_file: &String, metric_obj: &Telemetry, sleep_time: u64, termination_rx: &mpsc::Receiver<()>) -> Result<()> {
    loop {
        match File::open(log_file).await {
            Ok(file) => {
                info!("Hasura log file {} open, will follow the log", log_file);
                match process_file(log_file, metric_obj, file, sleep_time, termination_rx).await {
                    Ok(true) => (),
                    Ok(false) => return Ok(()),
                    Err(e) => {
                        warn!("Error reading logfile: {}", e);
                        ()
                    }
                };
                info!("Need to reopen hasura log file {}", log_file);
            }
            Err(e) => {
                error!("File {} could not be opened ({}). Will wait a little and then try again...", log_file, e);
                match termination_rx.recv_timeout(std::time::Duration::from_millis(sleep_time)) {
                    Ok(_) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
                    Err(RecvTimeoutError::Timeout) => () //continue
                }
            }
        }
    }
}

async fn process_file(file_name: &String, metric_obj: &Telemetry, file: File, sleep_time: u64, termination_rx: &mpsc::Receiver<()>) -> Result<bool> {
    let (watch_sender, watch_receiver) = mpsc::channel();
    let mut watcher = watcher(watch_sender, Duration::from_secs(1)).unwrap();
    watcher.watch(file_name, RecursiveMode::NonRecursive).unwrap();

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    loop {
        match watch_receiver.recv_timeout(Duration::from_millis(sleep_time)) {
            Ok(DebouncedEvent::Write(_)) => (), //something was written to the file, read it
            Ok(DebouncedEvent::Remove(_)) => {
                info!("hasura logfile was removed");
                return Ok(true);
            },
            Ok(DebouncedEvent::Rename(_, _)) => {
                info!("hasura logfile was renamed");
                return Ok(true);
            },
            Ok(DebouncedEvent::Chmod(_)) => { //happens when moving another file to it
                info!("hasura logfile was chmod'ed");
                return Ok(true);
            },
            Ok(DebouncedEvent::Rescan) => {
                info!("hasura logfile needs rescan");
                return Ok(true);
            },
            Ok(DebouncedEvent::Error(e, _)) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Watching error {}", e)));
            },
            Ok(_) => (),
            Err(RecvTimeoutError::Disconnected) => return Ok(true),
            Err(RecvTimeoutError::Timeout) => ()
        }

        // read data as long as there's new data available
        loop {
            match termination_rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    return Ok(false)
                },
                Err(TryRecvError::Empty) => () //continue
            }

            if let Some(line) = lines.next_line().await? {
                logprocessor::log_processor(&line,metric_obj).await;
            } else {
                break;
            }
        }
    }
}
