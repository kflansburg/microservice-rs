use config::ConfigError as Error;
use super::logging::Config as LoggingConfig;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig<Config> {
    pub logging: Option<LoggingConfig>,
    #[serde(flatten)]
    pub app: Config
}

pub fn load_config<Config: serde::de::DeserializeOwned>() -> Result<AppConfig<Config>, Error> {
    let mut s = config::Config::new();
    s.merge(config::File::with_name("Config.yaml").required(false))?;
    s.merge(config::Environment::with_prefix("APP").separator("_"))?;
    Ok(s.try_into::<AppConfig<Config>>()?)
}
