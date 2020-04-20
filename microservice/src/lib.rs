use slog::{o, info};

pub use microservice_macro::main;
pub use handler::Signal;

mod handler;
mod logging;
mod config;

pub fn microservice_run<Config, Output>(app_main: fn(logger: slog::Logger, config: Config, signal: Signal) -> Output) -> Output
    where Config: serde::de::DeserializeOwned + std::fmt::Debug {

    let config = config::load_config().expect("Unable to load application configuration.");
    let (root_logger, guard) = logging::logger(config.logging.as_ref()).expect("Unable to open file for logging.");
    let running = handler::set_handler(root_logger.new(o!("component" => "signal_handler"))).expect("Unable to set signal handler.");

    info!(root_logger, "Application Starting.";
        "config" => format!("{:?}", &config)
    );

    let logger = root_logger.new(o!("component" => "main"));
    let result = app_main(logger, config.app, running);

    info!(root_logger, "Application Exiting.");
    drop(guard);
    result
}
