use crate::database::get_pool;
use crate::models::{EnergyConsumption, PredictionResult};
use crate::services::EnergyPredictor;
use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PredictionQuery {
    pub device_id: String,
    pub hours: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct PredictionsResponse {
    pub prediction: PredictionResult,
    pub generated_at: DateTime<Utc>,
}

pub fn router() -> Router {
    Router::new().route("/", get(get_predictions))
}

async fn get_predictions(
    Query(query): Query<PredictionQuery>,
) -> Result<Json<PredictionsResponse>, (StatusCode, String)> {
    let pool = get_pool();
    
    let forecast_hours = query.hours.unwrap_or(24);
    
    // Fetch historical data for the device
    let cutoff_time = Utc::now() - Duration::hours(720); // Get up to 30 days of history
    
    let rows = sqlx::query_as::<_, EnergyConsumption>(
        r#"
        SELECT id, device_id, kwh, timestamp, created_at 
        FROM energy_consumption 
        WHERE device_id = ? AND timestamp > ? 
        ORDER BY timestamp ASC
        "#,
    )
    .bind(&query.device_id)
    .bind(cutoff_time)
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    // Convert to (timestamp, kwh) pairs
    let history: Vec<(DateTime<Utc>, f64)> = rows
        .into_iter()
        .map(|c| (c.timestamp, c.kwh))
        .collect();

    // Generate predictions using ML predictor
    let predictor = EnergyPredictor::new(0.8);
    let prediction = predictor.predict(&query.device_id, history, forecast_hours).await;

    Ok(Json(PredictionsResponse {
        prediction,
        generated_at: Utc::now(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prediction_query_validation() {
        let query = PredictionQuery {
            device_id: "test_device".to_string(),
            hours: Some(24),
        };

        assert!(!query.device_id.is_empty());
        assert!(query.hours.unwrap_or(24) > 0);
    }
}
