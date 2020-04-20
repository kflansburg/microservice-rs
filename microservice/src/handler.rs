use {
    std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering}
    },
    slog::{Logger, warn}
};

#[derive(Clone)]
pub struct Signal {
    running: Arc<AtomicBool>
}

impl Signal {
    /// Check if a signal has been caught and the application should exit. 
    pub fn check(&self) -> bool {
        !self.running.load(Ordering::Relaxed) 
    }
}

pub fn set_handler(logger: Logger) -> Result<Signal, ctrlc::Error> {
    let running = Arc::new(AtomicBool::new(true));
    let handler_running = running.clone();
    let handler = move || {
        warn!(logger, "Got Signal.");
        handler_running.store(false, Ordering::Relaxed);
    };
    ctrlc::set_handler(handler)?;
    Ok(Signal { running })
}
