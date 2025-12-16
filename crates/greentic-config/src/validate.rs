use greentic_config_types::{GreenticConfig, PackSourceConfig, TelemetryExporterKind};
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
