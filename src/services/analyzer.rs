use crate::models::{ConsumptionSummary, HourlyConsumption};
use chrono::{DateTime, Duration, Utc};
use tracing::info;

/// Analyzes energy consumption patterns and trends
pub struct EnergyAnalyzer;

impl EnergyAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Calculate summary statistics for a device's consumption
    pub async fn analyze(&self, readings: Vec<(DateTime<Utc>, f64)>) -> Option<ConsumptionSummary> {
        if readings.is_empty() {
            return None;
        }

        let kwh_values: Vec<f64> = readings.iter().map(|(_, kwh)| *kwh).collect();
        
        let total_kwh: f64 = kwh_values.iter().sum();
        let count = kwh_values.len() as u32;
        let avg_kwh = total_kwh / count as f64;
        
        let max_kwh = *kwh_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_kwh = *kwh_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        // Find the most common device_id (assuming all readings are for same device)
        let device_id = "unknown".to_string();

        Some(ConsumptionSummary {
            device_id,
            total_kwh: (total_kwh * 1000.0).round() / 1000.0,
            avg_kwh: (avg_kwh * 1000.0).round() / 1000.0,
            max_kwh: (max_kwh * 1000.0).round() / 1000.0,
            min_kwh: (min_kwh * 1000.0).round() / 1000.0,
            reading_count: count,
        })
    }

    /// Group consumption data by hour of day
    pub fn group_by_hour(&self, readings: Vec<(DateTime<Utc>, f64)>) -> Vec<HourlyConsumption> {
        let mut hourly_map: std::collections::HashMap<i32, (f64, u32)> = std::collections::HashMap::new();

        for (timestamp, kwh) in readings {
            let hour = timestamp.hour() as i32;
            let entry = hourly_map.entry(hour).or_insert((0.0, 0));
            entry.0 += kwh;
            entry.1 += 1;
        }

        hourly_map
            .into_iter()
            .map(|(hour, (total, count))| HourlyConsumption {
                hour: Utc::now().with_hour(hour as u32).unwrap_or_else(Utc::now),
                kwh: total / count as f64,
            })
            .collect()
    }

    /// Calculate trend (increasing/decreasing/stable)
    pub fn calculate_trend(&self, readings: Vec<(DateTime<Utc>, f64)>) -> Trend {
        if readings.len() < 2 {
            return Trend::Stable;
        }

        // Simple linear regression to determine trend
        let n = readings.len() as f64;
        let x_sum: f64 = (0..readings.len()).map(|i| i as f64).sum();
        let y_sum: f64 = readings.iter().map(|(_, kwh)| *kwh).sum();
        
        let xy_sum: f64 = readings
            .iter()
            .enumerate()
            .map(|(i, (_, kwh))| i as f64 * *kwh)
            .sum();
        
        let x_sq_sum: f64 = (0..readings.len()).map(|i| (i as f64).powi(2)).sum();

        let denominator = n * x_sq_sum - x_sum.powi(2);
        
        if denominator.abs() < 1e-10 {
            return Trend::Stable;
        }

        let slope = (n * xy_sum - x_sum * y_sum) / denominator;
        
        // Calculate mean for relative comparison
        let mean_y = y_sum / n;
        let relative_slope = if mean_y.abs() > 1e-10 { slope / mean_y } else { slope };

        if relative_slope > 0.05 {
            Trend::Increasing
        } else if relative_slope < -0.05 {
            Trend::Decreasing
        } else {
            Trend::Stable
        }
    }

    /// Compare consumption between two periods
    pub fn compare_periods(
        &self,
        period1: Vec<(DateTime<Utc>, f64)>,
        period2: Vec<(DateTime<Utc>, f64)>,
    ) -> PeriodComparison {
        let avg1 = if period1.is_empty() {
            0.0
        } else {
            period1.iter().map(|(_, kwh)| *kwh).sum::<f64>() / period1.len() as f64
        };

        let avg2 = if period2.is_empty() {
            0.0
        } else {
            period2.iter().map(|(_, kwh)| *kwh).sum::<f64>() / period2.len() as f64
        };

        let change_percent = if avg1.abs() > 1e-10 {
            ((avg2 - avg1) / avg1) * 100.0
        } else {
            0.0
        };

        PeriodComparison {
            period1_avg_kwh: (avg1 * 1000.0).round() / 1000.0,
            period2_avg_kwh: (avg2 * 1000.0).round() / 1000.0,
            change_percent,
            is_improving: change_percent < 0.0, // Lower consumption is better
        }
    }

    /// Identify the top N highest consumption periods
    pub fn identify_peaks(&self, readings: Vec<(DateTime<Utc>, f64)>, n: usize) -> Vec<(DateTime<Utc>, f64)> {
        let mut sorted = readings.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted.into_iter().take(n).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    pub period1_avg_kwh: f64,
    pub period2_avg_kwh: f64,
    pub change_percent: f64,
    pub is_improving: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_summary() {
        let analyzer = EnergyAnalyzer::new();
        
        let readings = vec![
            (Utc::now(), 2.5),
            (Utc::now() - Duration::hours(1), 3.0),
            (Utc::now() - Duration::hours(2), 2.8),
        ];

        let summary = analyzer.analyze(readings).await;
        
        assert!(summary.is_some());
        let s = summary.unwrap();
        assert_eq!(s.reading_count, 3);
        assert!((s.avg_kwh - 2.767).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_trend_increasing() {
        let analyzer = EnergyAnalyzer::new();
        
        let readings = vec![
            (Utc::now(), 5.0),
            (Utc::now() - Duration::hours(1), 4.0),
            (Utc::now() - Duration::hours(2), 3.0),
            (Utc::now() - Duration::hours(3), 2.0),
        ];

        let trend = analyzer.calculate_trend(readings);
        
        assert_eq!(trend, Trend::Increasing);
    }

    #[tokio::test]
    async fn test_period_comparison() {
        let analyzer = EnergyAnalyzer::new();
        
        let period1 = vec![
            (Utc::now(), 3.0),
            (Utc::now() - Duration::hours(1), 3.0),
        ];
        
        let period2 = vec![
            (Utc::now(), 2.5),
            (Utc::now() - Duration::hours(1), 2.5),
        ];

        let comparison = analyzer.compare_periods(period1, period2);
        
        assert!(comparison.is_improving); // Lower consumption is better
        assert!((comparison.change_percent - (-16.67)).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_identify_peaks() {
        let analyzer = EnergyAnalyzer::new();
        
        let readings = vec![
            (Utc::now(), 2.0),
            (Utc::now() - Duration::hours(1), 5.0),
            (Utc::now() - Duration::hours(2), 3.0),
            (Utc::now() - Duration::hours(3), 4.0),
        ];

        let peaks = analyzer.identify_peaks(readings, 2);
        
        assert_eq!(peaks.len(), 2);
        assert_eq!(peaks[0].1, 5.0);
        assert_eq!(peaks[1].1, 4.0);
    }
}
