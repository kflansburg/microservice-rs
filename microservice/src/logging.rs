use {
    slog::{self, Drain, o},
    serde::{self, Deserialize},
    std::str::FromStr
};

fn deserialize_level<'de, D>(deserializer: D) -> Result<Option<slog::Level>, D::Error> where D: serde::de::Deserializer<'de> {
    deserializer.deserialize_any(LevelVisitor)
}

struct LevelVisitor;

impl<'de> serde::de::Visitor<'de> for LevelVisitor {
    type Value = Option<slog::Level>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A slog::Level string representation or null.")
    }

    fn visit_str<E>(self, value: &str) -> Result<Option<slog::Level>, E>
        where E: serde::de::Error
    {
        Ok(Some(slog::Level::from_str(value).map_err(|e| E::custom(format!("Error parsing log level '{}': {:?}", value, e)))?))
    }

    fn visit_unit<E>(self) -> Result<Option<slog::Level>, E>
        where E: serde::de::Error
    {
        Ok(None)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="snake_case")]
enum LoggingFormat {
    Json,
    Text
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(deserialize_with="deserialize_level")]
    level: Option<slog::Level>,
    format: LoggingFormat,
    path: Option<String>
}

impl Default for &Config {
    fn default() -> Self {  
        &Config {
            level: Some(slog::Level::Info),
            format: LoggingFormat::Json,
            path: None
        }
    }
}

pub fn logger(opt_config: Option<&Config>) -> Result<(slog::Logger, slog_async::AsyncGuard), std::io::Error> {
    let config = opt_config.unwrap_or_default();
    let (async_drain, async_guard) = match config.format {
        LoggingFormat::Json => {
            match config.path {
                Some(ref path) => {
                    let file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
                    let drain = slog_json::Json::new(file).add_default_keys().build().fuse();
                    slog_async::Async::new(drain).build_with_guard()
                }, 
                None => {
                    let drain = slog_json::Json::new(std::io::stdout()).add_default_keys().build().fuse();
                    slog_async::Async::new(drain).build_with_guard()
                }
            }
        },
        LoggingFormat::Text => {
            match config.path {
                Some(ref path) => {
                    let file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
                    let decorator = slog_term::PlainSyncDecorator::new(file);
                    let drain = slog_term::FullFormat::new(decorator).build().fuse();
                    slog_async::Async::new(drain).build_with_guard()
                }, 
                None => {
                    let decorator = slog_term::TermDecorator::new().build();
                    let drain = slog_term::FullFormat::new(decorator).build().fuse();
                    slog_async::Async::new(drain).build_with_guard()
                }
            }
        },
    };
    let filter_drain = slog::LevelFilter::new(async_drain, config.level.unwrap_or(slog::Level::Info)).fuse();
    let root_logger = slog::Logger::root(filter_drain, o!());
    Ok((root_logger, async_guard))
}
