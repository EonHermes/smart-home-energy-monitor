use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EnergyConsumption {
    pub id: Uuid,
    pub device_id: String,
    pub kwh: f64,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewConsumption {
    pub device_id: String,
    pub kwh: f64,
    #[serde(default = "now")]
    pub timestamp: DateTime<Utc>,
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumptionSummary {
    pub device_id: String,
    pub total_kwh: f64,
    pub avg_kwh: f64,
    pub max_kwh: f64,
    pub min_kwh: f64,
    pub reading_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyConsumption {
    pub hour: DateTime<Utc>,
    pub kwh: f64,
}

impl EnergyConsumption {
    pub fn new(device_id: String, kwh: f64, timestamp: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            device_id,
            kwh,
            timestamp,
            created_at: Utc::now(),
        }
    }
}
