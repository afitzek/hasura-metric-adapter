use std::sync::mpsc::{Receiver, TryRecvError};

pub fn should_stop(rx: &Receiver<()>) -> bool {
    match rx.try_recv() {
        Ok(_) | Err(TryRecvError::Disconnected) => true,
        Err(TryRecvError::Empty) => false
    }
}
