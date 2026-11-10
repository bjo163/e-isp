use std::{env, error::Error, time::Duration};
use tokio::{signal, time};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let collector_id = env::var("COLLECTOR_ID").unwrap_or_else(|_| "local-unregistered".to_owned());
    if collector_id.trim().is_empty() {
        return Err("COLLECTOR_ID must not be empty".into());
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .json()
        .init();

    tracing::info!(
        collector_id = %collector_id,
        status = "bootstrap_only",
        "e-ISP collector runtime started; no device polling is enabled until an adapter profile is configured"
    );

    let mut heartbeat = time::interval(Duration::from_secs(30));
    heartbeat.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            signal_result = signal::ctrl_c() => {
                signal_result?;
                tracing::info!(collector_id = %collector_id, "collector shutdown requested");
                break;
            }
            _ = heartbeat.tick() => {
                tracing::debug!(collector_id = %collector_id, status = "bootstrap_only", "collector process is alive; registration/heartbeat transport is not configured yet");
            }
        }
    }

    Ok(())
}
