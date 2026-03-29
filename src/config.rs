use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub prediction: PredictionConfig,
    pub alerts: AlertConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    pub forecast_hours: u32,
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub high_usage_threshold_kwh: f64,
    pub notification_webhook: Option<String>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // Try to load from config.toml, otherwise use defaults
        if Path::new("config.toml").exists() {
            let cfg = config::Config::builder()
                .add_source(config::File::with_name("config"))
                .build()?;
            
            Ok(Config {
                server: ServerConfig {
                    host: cfg.get_string("server.host").unwrap_or_else(|_| "127.0.0.1".into()),
                    port: cfg.get_int("server.port").unwrap_or(3000) as u16,
                },
                database: DatabaseConfig {
                    path: cfg.get_string("database.path").unwrap_or_else(|_| "./energy_monitor.db".into()),
                },
                prediction: PredictionConfig {
                    forecast_hours: cfg.get_int("prediction.forecast_hours").unwrap_or(24) as u32,
                    confidence_threshold: cfg.get_float("prediction.confidence_threshold").unwrap_or(0.85),
                },
                alerts: AlertConfig {
                    high_usage_threshold_kwh: cfg.get_float("alerts.high_usage_threshold_kwh").unwrap_or(5.0),
                    notification_webhook: cfg.get_string("alerts.notification_webhook").ok(),
                },
            })
        } else {
            Ok(Self::default())
        }
    }

    pub fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                path: "./energy_monitor.db".to_string(),
            },
            prediction: PredictionConfig {
                forecast_hours: 24,
                confidence_threshold: 0.85,
            },
            alerts: AlertConfig {
                high_usage_threshold_kwh: 5.0,
                notification_webhook: None,
            },
        }
    }
}
