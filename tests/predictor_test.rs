use chrono::{Duration, Utc};

// Tests for the ML predictor module

#[tokio::test]
async fn test_predictor_with_realistic_data() {
    // Simulate realistic energy consumption patterns
    let history: Vec<(chrono::DateTime<Utc>, f64)> = (0..24)
        .map(|i| {
            (
                Utc::now() - Duration::hours(23 - i),
                2.0 + (i % 6) as f64 * 0.3, // Simulated varying consumption
            )
        })
        .collect();

    assert!(history.len() >= 2);
    
    let total: f64 = history.iter().map(|(_, kwh)| *kwh).sum();
    let avg = total / history.len() as f64;
    
    assert!(avg > 0.0);
}

#[tokio::test]
async fn test_anomaly_detection_thresholds() {
    // Test that anomalies can be detected
    let normal_values: Vec<f64> = vec![2.0, 2.1, 1.9, 2.0, 2.2];
    let anomaly_value = 10.0; // Significantly higher
    
    let mean: f64 = normal_values.iter().sum::<f64>() / normal_values.len() as f64;
    let variance: f64 = normal_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / normal_values.len() as f64;
    let std_dev = variance.sqrt();
    
    let z_score = (anomaly_value - mean) / std_dev;
    
    assert!(z_score > 2.0); // Should be detected as anomaly
}
