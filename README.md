# Smart Home Energy Monitor ⚡

Track and optimize your home energy usage with ML-powered predictions and intelligent insights.

![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)

## Features

- 📊 **Real-time Energy Tracking** - Monitor consumption from smart meters and IoT devices
- 🔮 **ML-Powered Predictions** - Forecast future energy usage using time-series analysis
- 💡 **Optimization Suggestions** - Get actionable recommendations to reduce consumption
- 📈 **Trend Analysis** - Visualize patterns and identify high-consumption periods
- 🔔 **Smart Alerts** - Receive notifications when usage exceeds thresholds
- 🌐 **RESTful API** - Easy integration with home automation systems
- 💾 **Local Storage** - Privacy-first design with SQLite database

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/EonHermes/smart-home-energy-monitor.git
cd smart-home-energy-monitor

# Build and run
cargo build --release
./target/release/smart-home-energy-monitor
```

### Configuration

Create a `config.toml` file:

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
path = "./energy_monitor.db"

[prediction]
forecast_hours = 24
confidence_threshold = 0.85

[alerts]
high_usage_threshold_kwh = 5.0
notification_webhook = "http://localhost:9000/alerts"
```

### API Examples

**Record energy consumption:**
```bash
curl -X POST http://localhost:3000/api/consumption \
  -H "Content-Type: application/json" \
  -d '{"device_id": "main_meter", "kwh": 2.5, "timestamp": "2026-03-29T15:41:00Z"}'
```

**Get consumption history:**
```bash
curl http://localhost:3000/api/consumption?device_id=main_meter&hours=24
```

**Get predictions:**
```bash
curl http://localhost:3000/api/predictions?device_id=main_meter&hours=24
```

**Get optimization suggestions:**
```bash
curl http://localhost:3000/api/optimizations?device_id=main_meter
```

## Architecture

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── api/                 # HTTP API handlers
│   ├── mod.rs
│   ├── consumption.rs   # Consumption endpoints
│   ├── predictions.rs   # Prediction endpoints
│   └── optimizations.rs # Optimization suggestions
├── models/              # Data structures
│   ├── mod.rs
│   ├── consumption.rs   # Energy consumption records
│   └── prediction.rs    # ML prediction results
├── services/            # Business logic
│   ├── mod.rs
│   ├── predictor.rs     # ML forecasting engine
│   ├── optimizer.rs     # Suggestion generator
│   └── analyzer.rs      # Trend analysis
├── database/            # Database operations
│   ├── mod.rs
│   └── migrations.rs    # Schema migrations
└── utils/               # Utilities
    └── time.rs          # Time handling helpers
```

## ML Forecasting

The prediction engine uses:
- **Linear Regression** for trend analysis
- **Moving Averages** for smoothing
- **Statistical Anomaly Detection** to identify unusual patterns
- **Confidence Intervals** for prediction reliability

## Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Submit a pull request

## License

MIT - See LICENSE file for details.
