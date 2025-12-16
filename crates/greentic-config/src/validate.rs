use greentic_config_types::{
    BackoffConfig, GreenticConfig, PackSourceConfig, ServiceEndpointConfig, TelemetryExporterKind,
};
use greentic_types::{ConnectionKind, EnvId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("dev configuration is not permitted for non-dev env ({0}) without allow_dev")]
    DevNotAllowed(String),
    #[error("path must be absolute: {0}")]
    RelativePath(String),
    #[error("telemetry sampling must be between 0.0 and 1.0 (got {0})")]
    TelemetrySampling(f32),
    #[error("packs.source requires connectivity but environment.connection is offline")]
    PacksSourceOffline,
    #[error("events endpoint not allowed for offline connection: {0}")]
    EventsEndpointOffline(String),
    #[error("events backoff.initial_ms must be greater than 0 (got {0})")]
    EventsBackoffInitial(u64),
    #[error("events backoff.max_ms must be >= initial_ms (got max={max}, initial={initial})")]
    EventsBackoffMax { max: u64, initial: u64 },
    #[error("events backoff.multiplier must be finite and >= 1.0 (got {0})")]
    EventsBackoffMultiplier(f64),
}

pub fn validate_config(
    config: &GreenticConfig,
    allow_dev: bool,
) -> Result<Vec<String>, ValidationError> {
    let mut warnings = Vec::new();

    if let Some(dev) = &config.dev {
        if !allow_dev && !is_dev_env(&config.environment.env_id) {
            let env_label = env_id_label(&config.environment.env_id);
            return Err(ValidationError::DevNotAllowed(env_label));
        }
        if dev.default_team.is_none() {
            warnings.push("dev.default_team not set; proceeding without team scoping".to_string());
        }
    }

    for path in [
        &config.paths.greentic_root,
        &config.paths.state_dir,
        &config.paths.cache_dir,
        &config.paths.logs_dir,
    ] {
        if !path.is_absolute() {
            return Err(ValidationError::RelativePath(path.display().to_string()));
        }
    }

    if !(0.0..=1.0).contains(&config.telemetry.sampling) {
        return Err(ValidationError::TelemetrySampling(
            config.telemetry.sampling,
        ));
    }

    if config.telemetry.enabled && matches!(config.telemetry.exporter, TelemetryExporterKind::None)
    {
        warnings
            .push("telemetry.enabled=true but exporter=none; telemetry will be disabled".into());
    }

    if let Some(packs) = &config.packs {
        ensure_absolute(&packs.cache_dir)?;
        match &packs.source {
            PackSourceConfig::LocalIndex { path } => ensure_absolute(path)?,
            PackSourceConfig::HttpIndex { .. } | PackSourceConfig::OciRegistry { .. } => {
                if matches!(config.environment.connection, Some(ConnectionKind::Offline)) {
                    return Err(ValidationError::PacksSourceOffline);
                }
            }
        }
    }

    if let Some(services) = &config.services
        && let Some(events) = &services.events
    {
        validate_events_endpoint(events, &config.environment.connection)?;
    }

    if let Some(events) = &config.events
        && let Some(backoff) = &events.backoff
    {
        validate_backoff(backoff)?;
    }

    Ok(warnings)
}

fn is_dev_env(env: &EnvId) -> bool {
    env_id_label(env).to_ascii_lowercase().contains("dev")
}

fn env_id_label(env: &EnvId) -> String {
    serde_json::to_value(env)
        .map(|v| v.to_string())
        .unwrap_or_else(|_| format!("{env:?}"))
}

fn ensure_absolute(path: &std::path::Path) -> Result<(), ValidationError> {
    if path.is_absolute() {
        Ok(())
    } else {
        Err(ValidationError::RelativePath(path.display().to_string()))
    }
}

fn validate_events_endpoint(
    endpoint: &ServiceEndpointConfig,
    connection: &Option<ConnectionKind>,
) -> Result<(), ValidationError> {
    if matches!(connection, Some(ConnectionKind::Offline)) && !is_local_url(&endpoint.url) {
        return Err(ValidationError::EventsEndpointOffline(
            endpoint.url.to_string(),
        ));
    }
    Ok(())
}

fn is_local_url(url: &url::Url) -> bool {
    match url.host_str() {
        Some("localhost") => true,
        Some(host) => host
            .parse::<std::net::IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false),
        None => false,
    }
}

fn validate_backoff(backoff: &BackoffConfig) -> Result<(), ValidationError> {
    if let Some(initial) = backoff.initial_ms
        && initial == 0
    {
        return Err(ValidationError::EventsBackoffInitial(initial));
    }
    if let (Some(max), Some(initial)) = (backoff.max_ms, backoff.initial_ms)
        && max < initial
    {
        return Err(ValidationError::EventsBackoffMax { max, initial });
    }
    if let Some(multiplier) = backoff.multiplier
        && (!multiplier.is_finite() || multiplier < 1.0)
    {
        return Err(ValidationError::EventsBackoffMultiplier(multiplier));
    }
    Ok(())
}
