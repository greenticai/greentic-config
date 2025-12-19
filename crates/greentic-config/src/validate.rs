use greentic_config_types::{
    BackoffConfig, GreenticConfig, PackSourceConfig, ServiceEndpointConfig, ServiceTransportConfig,
    TelemetryExporterKind,
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
    #[error("deployer.base_domain must be a valid DNS name (got {0})")]
    DeployerBaseDomain(String),
}

pub fn validate_config(
    config: &GreenticConfig,
    allow_dev: bool,
) -> Result<Vec<String>, ValidationError> {
    validate_config_with_overrides(config, allow_dev, false)
}

pub fn validate_config_with_overrides(
    config: &GreenticConfig,
    allow_dev: bool,
    allow_network: bool,
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

    if !allow_network
        && matches!(config.environment.connection, Some(ConnectionKind::Offline))
        && telemetry_endpoint_is_non_loopback(config.telemetry.endpoint.as_deref())
    {
        warnings.push(
            "environment.connection=offline but telemetry.endpoint is non-loopback; telemetry may attempt outbound network"
                .into(),
        );
    }

    if let Some(packs) = &config.packs {
        ensure_absolute(&packs.cache_dir)?;
        match &packs.source {
            PackSourceConfig::LocalIndex { path } => ensure_absolute(path)?,
            PackSourceConfig::HttpIndex { .. } | PackSourceConfig::OciRegistry { .. } => {
                if !allow_network
                    && matches!(config.environment.connection, Some(ConnectionKind::Offline))
                {
                    return Err(ValidationError::PacksSourceOffline);
                }
            }
        }
    }

    if let Some(services) = &config.services
        && let Some(events) = &services.events
        && !allow_network
    {
        validate_events_endpoint(events, &config.environment.connection)?;
    }

    if let Some(services) = &config.services
        && !allow_network
        && matches!(config.environment.connection, Some(ConnectionKind::Offline))
    {
        warn_offline_transport(&mut warnings, "runner", services.runner.as_ref());
        warn_offline_transport(&mut warnings, "deployer", services.deployer.as_ref());
        warn_offline_transport(
            &mut warnings,
            "events_transport",
            services.events_transport.as_ref(),
        );
        warn_offline_transport(&mut warnings, "source", services.source.as_ref());
        warn_offline_transport(&mut warnings, "publish", services.publish.as_ref());
        warn_offline_transport(&mut warnings, "metadata", services.metadata.as_ref());
        warn_offline_transport(
            &mut warnings,
            "oauth_broker",
            services.oauth_broker.as_ref(),
        );
    }

    if let Some(events) = &config.events
        && let Some(backoff) = &events.backoff
    {
        validate_backoff(backoff)?;
    }

    if let Some(deployer) = &config.deployer
        && let Some(base_domain) = &deployer.base_domain
    {
        validate_base_domain(base_domain)?;
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

fn telemetry_endpoint_is_non_loopback(endpoint: Option<&str>) -> bool {
    let Some(raw) = endpoint else {
        return false;
    };
    let Ok(url) = url::Url::parse(raw) else {
        return true;
    };
    !is_local_url(&url)
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

fn validate_base_domain(domain: &str) -> Result<(), ValidationError> {
    let trimmed = domain.trim();
    if trimmed.is_empty()
        || trimmed.contains("://")
        || trimmed.contains('/')
        || trimmed.contains(' ')
    {
        return Err(ValidationError::DeployerBaseDomain(domain.to_string()));
    }

    let labels = trimmed.split('.').collect::<Vec<_>>();
    if labels.iter().any(|label| label.is_empty()) {
        return Err(ValidationError::DeployerBaseDomain(domain.to_string()));
    }

    for label in labels {
        if label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(ValidationError::DeployerBaseDomain(domain.to_string()));
        }
    }

    Ok(())
}

fn warn_offline_transport(
    warnings: &mut Vec<String>,
    name: &str,
    transport: Option<&ServiceTransportConfig>,
) {
    let Some(transport) = transport else {
        return;
    };
    match transport {
        ServiceTransportConfig::Noop => {}
        ServiceTransportConfig::Http { url, .. } | ServiceTransportConfig::Nats { url, .. } => {
            if !is_local_url(url) {
                warnings.push(format!(
                    "environment.connection=offline but services.{name} transport configured at {url}; network operations may be disallowed"
                ));
            }
        }
    }
}
