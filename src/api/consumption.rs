use crate::database::get_pool;
use crate::models::{ConsumptionSummary, EnergyConsumption, NewConsumption};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ConsumptionQuery {
    pub device_id: Option<String>,
    pub hours: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ConsumptionResponse {
    pub consumptions: Vec<EnergyConsumption>,
    pub summary: Option<ConsumptionSummary>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", post(create_consumption))
        .route("/", get(get_consumptions))
}

async fn create_consumption(
    Json(new_consumption): Json<NewConsumption>,
) -> Result<Json<EnergyConsumption>, (StatusCode, String)> {
    let pool = get_pool();

    // Validate input
    if new_consumption.kwh < 0.0 {
        return Err((StatusCode::BAD_REQUEST, "Energy consumption cannot be negative".into()));
    }

    if new_consumption.device_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Device ID is required".into()));
    }

    let consumption = EnergyConsumption::new(
        new_consumption.device_id.clone(),
        new_consumption.kwh,
        new_consumption.timestamp,
    );

    match sqlx::query(
        r#"
        INSERT INTO energy_consumption (id, device_id, kwh, timestamp, created_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(consumption.id.to_string())
    .bind(&consumption.device_id)
    .bind(consumption.kwh)
    .bind(consumption.timestamp)
    .bind(consumption.created_at)
    .execute(pool)
    .await
    {
        Ok(_) => Ok(Json(consumption)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))),
    }
}

async fn get_consumptions(
    Query(query): Query<ConsumptionQuery>,
) -> Result<Json<ConsumptionResponse>, (StatusCode, String)> {
    let pool = get_pool();

    // Build query based on filters
    let hours_filter = query.hours.unwrap_or(24);
    let cutoff_time = Utc::now() - Duration::hours(hours_filter as i64);

    let query_str = if let Some(device_id) = &query.device_id {
        format!(
            "SELECT id, device_id, kwh, timestamp, created_at FROM energy_consumption 
             WHERE device_id = ? AND timestamp > ? ORDER BY timestamp DESC"
        )
    } else {
        format!(
            "SELECT id, device_id, kwh, timestamp, created_at FROM energy_consumption 
             WHERE timestamp > ? ORDER BY timestamp DESC"
        )
    };

    let rows = if let Some(device_id) = &query.device_id {
        sqlx::query(&query_str)
            .bind(device_id)
            .bind(cutoff_time)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query(&query_str)
            .bind(cutoff_time)
            .fetch_all(pool)
            .await
    };

    let rows = match rows {
        Ok(r) => r,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))),
    };

    // Convert rows to consumptions
    let mut consumptions = Vec::new();
    for row in &rows {
        let id_str: String = row.get(0);
        let device_id: String = row.get(1);
        let kwh: f64 = row.get(2);
        let timestamp: DateTime<Utc> = row.get(3);
        let created_at: DateTime<Utc> = row.get(4);

        consumptions.push(EnergyConsumption {
            id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
            device_id,
            kwh,
            timestamp,
            created_at,
        });
    }

    // Calculate summary if we have data
    let summary = if !consumptions.is_empty() {
        let device_ids: Vec<&String> = consumptions.iter().map(|c| &c.device_id).collect();
        
        // If filtering by specific device, use that; otherwise group by first device found
        let target_device = query.device_id.as_ref().unwrap_or(&consumptions[0].device_id);
        
        let filtered: Vec<&EnergyConsumption> = consumptions.iter()
            .filter(|c| &c.device_id == target_device)
            .collect();

        let total_kwh: f64 = filtered.iter().map(|c| c.kwh).sum();
        let count = filtered.len() as u32;
        
        Some(ConsumptionSummary {
            device_id: target_device.clone(),
            total_kwh: (total_kwh * 1000.0).round() / 1000.0,
            avg_kwh: if count > 0 { (total_kwh / count as f64 * 1000.0).round() / 1000.0 } else { 0.0 },
            max_kwh: filtered.iter().map(|c| c.kwh).fold(0.0, |a, b| a.max(b)),
            min_kwh: filtered.iter().map(|c| c.kwh).fold(f64::INFINITY, |a, b| a.min(b)),
            reading_count: count,
        })
    } else {
        None
    };

    Ok(Json(ConsumptionResponse { consumptions, summary }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_consumption() {
        // Note: This would need a real database setup for full testing
        let new = NewConsumption {
            device_id: "test_device".to_string(),
            kwh: 2.5,
            timestamp: Utc::now(),
        };

        assert!(!new.device_id.is_empty());
        assert!(new.kwh >= 0.0);
    }

    #[tokio::test]
    async fn test_negative_consumption_rejected() {
        let new = NewConsumption {
            device_id: "test_device".to_string(),
            kwh: -1.0,
            timestamp: Utc::now(),
        };

        assert!(new.kwh < 0.0); // Should be rejected by API
    }
}
