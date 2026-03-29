use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyPrediction {
    pub id: Uuid,
    pub device_id: String,
    pub predicted_kwh: f64,
    pub confidence: f64,
    pub prediction_for: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub device_id: String,
    pub predictions: Vec<HourlyPrediction>,
    pub summary: PredictionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyPrediction {
    pub timestamp: DateTime<Utc>,
    pub predicted_kwh: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionSummary {
    pub total_predicted_kwh: f64,
    pub avg_hourly_kwh: f64,
    pub peak_hour: Option<DateTime<Utc>>,
    pub peak_kwh: Option<f64>,
    pub confidence_avg: f64,
}

impl EnergyPrediction {
    pub fn new(
        device_id: String,
        predicted_kwh: f64,
        confidence: f64,
        prediction_for: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            device_id,
            predicted_kwh,
            confidence,
            prediction_for,
            created_at: Utc::now(),
        }
    }
}
