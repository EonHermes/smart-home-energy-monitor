use crate::models::{ConsumptionSummary, HourlyConsumption};
use chrono::{DateTime, Utc};
use tracing::info;

/// Generates optimization suggestions based on consumption patterns
pub struct EnergyOptimizer {
    high_usage_threshold: f64,
}

impl EnergyOptimizer {
    pub fn new(high_usage_threshold: f64) -> Self {
        Self {
            high_usage_threshold,
        }
    }

    /// Generate optimization suggestions based on consumption data
    pub async fn optimize(
        &self,
        device_id: &str,
        summary: ConsumptionSummary,
        hourly_data: Vec<HourlyConsumption>,
    ) -> Vec<OptimizationSuggestion> {
        info!("Generating optimization suggestions for device {}", device_id);

        let mut suggestions = Vec::new();

        // Check for high overall usage
        if summary.avg_kwh > self.high_usage_threshold {
            suggestions.push(OptimizationSuggestion {
                category: "High Usage".to_string(),
                priority: Priority::High,
                title: "Average consumption exceeds threshold".to_string(),
                description: format!(
                    "Your average hourly usage ({:.2} kWh) exceeds the recommended threshold ({:.2} kWh).",
                    summary.avg_kwh, self.high_usage_threshold
                ),
                recommendations: vec![
                    "Review which devices are consuming the most energy".to_string(),
                    "Consider upgrading to more energy-efficient appliances".to_string(),
                    "Implement smart scheduling for high-consumption devices".to_string(),
                ],
                potential_savings_kwh: Some(summary.avg_kwh * 0.2), // Estimate 20% savings
            });
        }

        // Analyze peak hours
        if let Some(peak) = hourly_data.iter().max_by(|a, b| a.kwh.partial_cmp(&b.kwh).unwrap()) {
            if peak.kwh > summary.avg_kwh * 1.5 {
                suggestions.push(OptimizationSuggestion {
                    category: "Peak Usage".to_string(),
                    priority: Priority::Medium,
                    title: "Significant peak consumption detected".to_string(),
                    description: format!(
                        "Peak usage at {:?} is {:.0}% above average. Consider load balancing.",
                        peak.hour.time(),
                        ((peak.kwh / summary.avg_kwh) - 1.0) * 100.0
                    ),
                    recommendations: vec![
                        "Shift non-essential tasks to off-peak hours".to_string(),
                        "Use smart plugs to automate device scheduling".to_string(),
                        "Consider battery storage for peak shaving".to_string(),
                    ],
                    potential_savings_kwh: Some((peak.kwh - summary.avg_kwh) * 0.3),
                });
            }
        }

        // Check for variability
        let std_dev = self.calculate_std_dev(&hourly_data, summary.avg_kwh);
        if std_dev > summary.avg_kwh * 0.5 {
            suggestions.push(OptimizationSuggestion {
                category: "High Variability".to_string(),
                priority: Priority::Medium,
                title: "Inconsistent consumption patterns detected".to_string(),
                description: format!(
                    "Your energy usage varies significantly (std dev: {:.2} kWh). Consistent usage is more efficient.",
                    std_dev
                ),
                recommendations: vec![
                    "Establish regular usage schedules for major appliances".to_string(),
                    "Use timers and automation to smooth consumption".to_string(),
                    "Review intermittent high-consumption activities".to_string(),
                ],
                potential_savings_kwh: Some(summary.avg_kwh * 0.1),
            });
        }

        // Check for weekend vs weekday patterns (if we have enough data)
        if hourly_data.len() >= 48 {
            self.analyze_weekday_weekend(&hourly_data, &mut suggestions);
        }

        // General energy-saving tips
        suggestions.push(OptimizationSuggestion {
            category: "General Tips".to_string(),
            priority: Priority::Low,
            title: "Energy efficiency recommendations".to_string(),
            description: "Here are some general tips to reduce your energy consumption:".to_string(),
            recommendations: vec![
                "Unplug devices when not in use (phantom load can account for 10% of usage)".to_string(),
                "Use LED lighting instead of incandescent bulbs".to_string(),
                "Set thermostat to 68°F (20°C) in winter, 78°F (26°C) in summer".to_string(),
                "Regular maintenance of HVAC systems improves efficiency by up to 15%".to_string(),
                "Consider smart home automation for optimal energy management".to_string(),
            ],
            potential_savings_kwh: None,
        });

        // Sort by priority
        suggestions.sort_by(|a, b| a.priority.cmp(&b.priority));

        suggestions
    }

    fn calculate_std_dev(&self, hourly_data: &[HourlyConsumption], mean: f64) -> f64 {
        if hourly_data.is_empty() {
            return 0.0;
        }

        let variance: f64 = hourly_data
            .iter()
            .map(|h| (h.kwh - mean).powi(2))
            .sum::<f64>() / hourly_data.len() as f64;

        variance.sqrt()
    }

    fn analyze_weekday_weekend(
        &self,
        hourly_data: &[HourlyConsumption],
        suggestions: &mut Vec<OptimizationSuggestion>,
    ) {
        // Simple analysis - in production, would use proper date parsing
        let total_kwh: f64 = hourly_data.iter().map(|h| h.kwh).sum();
        let avg_kwh = total_kwh / hourly_data.len() as f64;

        // Check if there's a pattern (simplified - just checking variance)
        let high_hours: usize = hourly_data.iter().filter(|h| h.kwh > avg_kwh * 1.3).count();
        
        if high_hours > hourly_data.len() / 4 {
            suggestions.push(OptimizationSuggestion {
                category: "Usage Patterns".to_string(),
                priority: Priority::Low,
                title: "Identify your high-consumption periods".to_string(),
                description: format!(
                    "{} hours show significantly elevated consumption. Review what activities occur during these times.",
                    high_hours
                ),
                recommendations: vec![
                    "Track which devices are active during peak hours".to_string(),
                    "Consider time-of-use pricing and shift usage accordingly".to_string(),
                    "Set up automated alerts for unusual consumption spikes".to_string(),
                ],
                potential_savings_kwh: Some(avg_kwh * 0.15),
            });
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub recommendations: Vec<String>,
    pub potential_savings_kwh: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimizer_generates_suggestions() {
        let optimizer = EnergyOptimizer::new(5.0);
        
        let summary = ConsumptionSummary {
            device_id: "test".to_string(),
            total_kwh: 120.0,
            avg_kwh: 5.0,
            max_kwh: 8.0,
            min_kwh: 2.0,
            reading_count: 24,
        };

        let hourly_data = (0..24)
            .map(|i| HourlyConsumption {
                hour: Utc::now() - Duration::hours(23 - i),
                kwh: 2.0 + (i % 6) as f64 * 0.5,
            })
            .collect();

        let suggestions = optimizer.optimize("test", summary, hourly_data).await;
        
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.category == "General Tips"));
    }

    #[tokio::test]
    async fn test_high_usage_suggestion() {
        let optimizer = EnergyOptimizer::new(3.0);
        
        let summary = ConsumptionSummary {
            device_id: "test".to_string(),
            total_kwh: 120.0,
            avg_kwh: 5.0, // Above threshold of 3.0
            max_kwh: 8.0,
            min_kwh: 2.0,
            reading_count: 24,
        };

        let hourly_data = vec![];

        let suggestions = optimizer.optimize("test", summary, hourly_data).await;
        
        assert!(suggestions.iter().any(|s| s.category == "High Usage"));
    }
}
