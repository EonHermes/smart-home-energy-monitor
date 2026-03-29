use crate::models::{HourlyPrediction, PredictionResult, PredictionSummary};
use chrono::{DateTime, Duration, Utc};
use ndarray::{Array1, Array2};
use statrs::distribution::{LinearRegression, Normal};
use statrs::prelude::*;
use std::collections::HashMap;
use tracing::info;

/// ML-powered energy consumption predictor using linear regression and statistical analysis
pub struct EnergyPredictor {
    confidence_threshold: f64,
}

impl EnergyPredictor {
    pub fn new(confidence_threshold: f64) -> Self {
        Self { confidence_threshold }
    }

    /// Generate predictions for the next N hours based on historical data
    pub async fn predict(
        &self,
        device_id: &str,
        history: Vec<(DateTime<Utc>, f64)>,
        forecast_hours: u32,
    ) -> PredictionResult {
        if history.len() < 2 {
            // Not enough data for prediction
            return self.empty_prediction(device_id, forecast_hours);
        }

        info!("Generating {}-hour forecast for device {}", forecast_hours, device_id);

        // Prepare data for regression
        let (predictions, summary) = self.generate_predictions(&history, forecast_hours);

        PredictionResult {
            device_id: device_id.to_string(),
            predictions,
            summary,
        }
    }

    fn generate_predictions(
        &self,
        history: &[(DateTime<Utc>, f64)],
        forecast_hours: u32,
    ) -> (Vec<HourlyPrediction>, PredictionSummary) {
        // Convert timestamps to numeric values (hours since first reading)
        let start_time = history.first().unwrap().0;
        
        let x_values: Vec<f64> = history
            .iter()
            .map(|(ts, _)| (*ts - *start_time).num_hours() as f64)
            .collect();
        
        let y_values: Vec<f64> = history.iter().map(|(_, kwh)| *kwh).collect();

        // Perform linear regression
        let x_array = Array1::from(x_values.clone());
        let y_array = Array1::from(y_values.clone());

        // Calculate basic statistics
        let mean_x = x_array.mean().unwrap_or(0.0);
        let mean_y = y_array.mean().unwrap_or(0.0);
        
        let variance_x: f64 = x_array.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>() / x_array.len() as f64;
        let variance_y: f64 = y_array.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>() / y_array.len() as f64;

        // Calculate covariance and slope
        let covariance: f64 = x_array
            .iter()
            .zip(y_array.iter())
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>() / x_array.len() as f64;

        let slope = if variance_x > 0.0 { covariance / variance_x } else { 0.0 };
        let intercept = mean_y - slope * mean_x;

        // Calculate R-squared (coefficient of determination)
        let ss_res: f64 = x_array
            .iter()
            .zip(y_array.iter())
            .map(|(x, y)| {
                let predicted = slope * x + intercept;
                (y - predicted).powi(2)
            })
            .sum();
        
        let ss_tot: f64 = y_array.iter().map(|y| (y - mean_y).powi(2)).sum();
        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

        // Calculate standard error for confidence intervals
        let std_error = if history.len() > 2 {
            (ss_res / (history.len() as f64 - 2.0)).sqrt()
        } else {
            variance_y.sqrt()
        };

        // Generate predictions for future hours
        let last_time = history.last().unwrap().0;
        let mut predictions = Vec::new();
        let mut total_predicted = 0.0;
        let mut peak_hour: Option<DateTime<Utc>> = None;
        let mut peak_kwh: Option<f64> = None;

        for hour_offset in 1..=forecast_hours {
            let predict_time = last_time + Duration::hours(hour_offset as i64);
            let x_pred = (*predict_time - *start_time).num_hours() as f64;
            
            let predicted_kwh = slope * x_pred + intercept;
            let confidence = self.calculate_confidence(r_squared, hour_offset as u32, history.len());
            
            // Confidence interval (95%)
            let z_score = 1.96;
            let margin = z_score * std_error;
            let lower_bound = (predicted_kwh - margin).max(0.0);
            let upper_bound = predicted_kwh + margin;

            predictions.push(HourlyPrediction {
                timestamp: predict_time,
                predicted_kwh: (predicted_kwh * 1000.0).round() / 1000.0,
                lower_bound: (lower_bound * 1000.0).round() / 1000.0,
                upper_bound: (upper_bound * 1000.0).round() / 1000.0,
                confidence,
            });

            total_predicted += predicted_kwh;
            
            if peak_kwh.map_or(true, |pk| predicted_kwh > pk) {
                peak_kwh = Some(predicted_kwh);
                peak_hour = Some(predict_time);
            }
        }

        let summary = PredictionSummary {
            total_predicted_kwh: (total_predicted * 1000.0).round() / 1000.0,
            avg_hourly_kwh: (total_predicted / forecast_hours as f64 * 1000.0).round() / 1000.0,
            peak_hour,
            peak_kwh: peak_kwh.map(|pk| (pk * 1000.0).round() / 1000.0),
            confidence_avg: r_squared,
        };

        (predictions, summary)
    }

    fn calculate_confidence(&self, r_squared: f64, forecast_horizon: u32, data_points: usize) -> f64 {
        // Base confidence from R-squared
        let base_confidence = r_squared;
        
        // Decay factor for forecast horizon (further predictions are less certain)
        let horizon_decay = 1.0 / (1.0 + (forecast_horizon as f64 * 0.05));
        
        // Data point bonus (more data = higher confidence)
        let data_bonus = (data_points as f64 / 100.0).min(1.0);
        
        let confidence = base_confidence * horizon_decay * (0.7 + 0.3 * data_bonus);
        confidence.clamp(0.0, 1.0)
    }

    fn empty_prediction(&self, device_id: &str, forecast_hours: u32) -> PredictionResult {
        let now = Utc::now();
        let predictions: Vec<HourlyPrediction> = (1..=forecast_hours)
            .map(|hour| HourlyPrediction {
                timestamp: now + Duration::hours(hour as i64),
                predicted_kwh: 0.0,
                lower_bound: 0.0,
                upper_bound: 0.0,
                confidence: 0.0,
            })
            .collect();

        PredictionResult {
            device_id: device_id.to_string(),
            predictions,
            summary: PredictionSummary {
                total_predicted_kwh: 0.0,
                avg_hourly_kwh: 0.0,
                peak_hour: None,
                peak_kwh: None,
                confidence_avg: 0.0,
            },
        }
    }

    /// Detect anomalies in consumption data using statistical methods
    pub fn detect_anomalies(&self, history: &[(DateTime<Utc>, f64)]) -> Vec<(DateTime<Utc>, f64, String)> {
        if history.len() < 3 {
            return vec![];
        }

        let values: Vec<f64> = history.iter().map(|(_, kwh)| *kwh).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let variance: f64 = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();

        history
            .iter()
            .filter_map(|(ts, kwh)| {
                let z_score = (kwh - mean) / std_dev;
                if z_score.abs() > 2.0 {
                    Some((
                        *ts,
                        *kwh,
                        if z_score > 0.0 {
                            "Unusually high consumption".to_string()
                        } else {
                            "Unusually low consumption".to_string()
                        },
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prediction_with_sufficient_data() {
        let predictor = EnergyPredictor::new(0.8);
        
        let history = vec![
            (Utc::now() - Duration::hours(24), 2.5),
            (Utc::now() - Duration::hours(12), 3.0),
            (Utc::now(), 2.8),
        ];

        let result = predictor.predict("test_device", history, 24).await;
        
        assert_eq!(result.device_id, "test_device");
        assert_eq!(result.predictions.len(), 24);
        assert!(result.summary.total_predicted_kwh > 0.0);
    }

    #[tokio::test]
    async fn test_prediction_with_insufficient_data() {
        let predictor = EnergyPredictor::new(0.8);
        
        let history = vec![(Utc::now(), 2.5)];

        let result = predictor.predict("test_device", history, 24).await;
        
        assert_eq!(result.predictions.len(), 24);
        assert_eq!(result.summary.total_predicted_kwh, 0.0);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let predictor = EnergyPredictor::new(0.8);
        
        let history = vec![
            (Utc::now() - Duration::hours(3), 2.5),
            (Utc::now() - Duration::hours(2), 2.6),
            (Utc::now() - Duration::hours(1), 2.4),
            (Utc::now(), 10.0), // Anomaly!
        ];

        let anomalies = predictor.detect_anomalies(&history);
        
        assert_eq!(anomalies.len(), 1);
        assert!(anomalies[0].2.contains("high"));
    }
}
