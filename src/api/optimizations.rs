use crate::database::get_pool;
use crate::models::{ConsumptionSummary, EnergyConsumption, HourlyConsumption};
use crate::services::EnergyOptimizer;
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
pub struct OptimizationQuery {
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct OptimizationsResponse {
    pub suggestions: Vec<OptimizationSuggestion>,
    pub summary: Option<ConsumptionSummary>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub recommendations: Vec<String>,
    pub potential_savings_kwh: Option<f64>,
}

pub fn router() -> Router {
    Router::new().route("/", get(get_optimizations))
}

async fn get_optimizations(
    Query(query): Query<OptimizationQuery>,
) -> Result<Json<OptimizationsResponse>, (StatusCode, String)> {
    let pool = get_pool();

    // Fetch recent consumption data (last 24 hours by default)
    let cutoff_time = Utc::now() - Duration::hours(24);

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

    if rows.is_empty() {
        return Ok(Json(OptimizationsResponse {
            suggestions: vec![OptimizationSuggestion {
                category: "No Data".to_string(),
                priority: "Low".to_string(),
                title: "Insufficient data for analysis".to_string(),
                description: format!(
                    "No consumption data found for device '{}' in the last 24 hours.",
                    query.device_id
                ),
                recommendations: vec![
                    "Start recording energy consumption data".to_string(),
                    "Ensure your smart meter or IoT devices are properly configured".to_string(),
                    "Wait at least 24 hours of data for meaningful insights".to_string(),
                ],
                potential_savings_kwh: None,
            }],
            summary: None,
            generated_at: Utc::now(),
        }));
    }

    // Calculate summary statistics
    let kwh_values: Vec<f64> = rows.iter().map(|r| r.kwh).collect();
    let total_kwh: f64 = kwh_values.iter().sum();
    let count = kwh_values.len() as u32;
    let avg_kwh = total_kwh / count as f64;
    
    let max_kwh = *kwh_values.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b));
    let min_kwh = *kwh_values.iter().fold(f64::INFINITY, |a, b| a.min(b));

    let summary = ConsumptionSummary {
        device_id: query.device_id.clone(),
        total_kwh: (total_kwh * 1000.0).round() / 1000.0,
        avg_kwh: (avg_kwh * 1000.0).round() / 1000.0,
        max_kwh: (max_kwh * 1000.0).round() / 1000.0,
        min_kwh: (min_kwh * 1000.0).round() / 1000.0,
        reading_count: count,
    };

    // Group by hour for pattern analysis
    let hourly_data: Vec<HourlyConsumption> = rows
        .iter()
        .map(|r| HourlyConsumption {
            hour: r.timestamp,
            kwh: r.kwh,
        })
        .collect();

    // Generate optimization suggestions
    let optimizer = EnergyOptimizer::new(5.0); // Default threshold of 5 kWh
    let suggestions = optimizer.optimize(&query.device_id, summary.clone(), hourly_data).await;

    // Convert to response format
    let suggestions: Vec<OptimizationSuggestion> = suggestions
        .into_iter()
        .map(|s| OptimizationSuggestion {
            category: s.category,
            priority: match s.priority {
                crate::services::Priority::High => "High".to_string(),
                crate::services::Priority::Medium => "Medium".to_string(),
                crate::services::Priority::Low => "Low".to_string(),
            },
            title: s.title,
            description: s.description,
            recommendations: s.recommendations,
            potential_savings_kwh: s.potential_savings_kwh.map(|v| (v * 1000.0).round() / 1000.0),
        })
        .collect();

    Ok(Json(OptimizationsResponse {
        suggestions,
        summary: Some(summary),
        generated_at: Utc::now(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimization_query_validation() {
        let query = OptimizationQuery {
            device_id: "test_device".to_string(),
        };

        assert!(!query.device_id.is_empty());
    }
}
