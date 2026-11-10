use loco_rs::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct SystemInfo {
    name: &'static str,
    version: &'static str,
    api_version: &'static str,
    status: &'static str,
    runtime: &'static str,
}

#[derive(Debug, Serialize)]
struct HealthStatus {
    status: &'static str,
    component: &'static str,
    readiness: &'static str,
}

async fn info() -> Result<Response> {
    format::json(SystemInfo {
        name: "e-ISP",
        version: env!("CARGO_PKG_VERSION"),
        api_version: "v1",
        status: "operational",
        runtime: "loco-rs",
    })
}

async fn health() -> Result<Response> {
    format::json(HealthStatus {
        status: "ok",
        component: "api-process",
        readiness: "process-only",
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/v1/system")
        .add("/info", get(info))
        .add("/health", get(health))
}
