use crate::paths::DefaultPaths;
use greentic_config_types::ConfigVersion;
use greentic_types::{ConnectionKind, DeploymentCtx, EnvId};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const ENV_PREFIX: &str = "GREENTIC_";
pub const DEFAULT_DEPLOYER_BASE_DOMAIN: &str = "deploy.greentic.ai";

#[derive(Debug, Clone, Copy)]
pub enum ConfigFileFormat {
    Toml,
    Json,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConfigLayer {
    #[serde(default)]
    pub schema_version: Option<ConfigVersion>,
    #[serde(default)]
    pub environment: Option<EnvironmentLayer>,
    #[serde(default)]
    pub paths: Option<PathsLayer>,
    #[serde(default)]
    pub services: Option<ServicesLayer>,
    #[serde(default)]
    pub events: Option<EventsLayer>,
    #[serde(default)]
    pub runtime: Option<RuntimeLayer>,
    #[serde(default)]
    pub telemetry: Option<TelemetryLayer>,
    #[serde(default)]
    pub network: Option<NetworkLayer>,
    #[serde(default)]
    pub deployer: Option<DeployerLayer>,
    #[serde(default)]
    pub secrets: Option<SecretsBackendRefLayer>,
    #[serde(default)]
    pub packs: Option<PacksLayer>,
    #[serde(default)]
    pub dev: Option<DevLayer>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentLayer {
    #[serde(default)]
    pub env_id: Option<EnvId>,
    #[serde(default)]
    pub deployment: Option<DeploymentCtx>,
    #[serde(default)]
    pub connection: Option<ConnectionKind>,
    #[serde(default)]
    pub region: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PathsLayer {
    #[serde(default)]
    pub greentic_root: Option<PathBuf>,
    #[serde(default)]
    pub state_dir: Option<PathBuf>,
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
    #[serde(default)]
    pub logs_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServicesLayer {
    #[serde(default)]
    pub events: Option<ServiceEndpointLayer>,
    #[serde(default)]
    pub runner: Option<ServiceLayer>,
    #[serde(default)]
    pub deployer: Option<ServiceLayer>,
    #[serde(default)]
    pub events_transport: Option<ServiceLayer>,
    #[serde(default)]
    pub source: Option<ServiceLayer>,
    #[serde(default)]
    pub publish: Option<ServiceLayer>,
    #[serde(default)]
    pub metadata: Option<ServiceLayer>,
    #[serde(default)]
    pub oauth_broker: Option<ServiceLayer>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceEndpointLayer {
    #[serde(default)]
    pub url: Option<url::Url>,
    #[serde(default)]
    pub headers: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EventsLayer {
    #[serde(default)]
    pub reconnect: Option<ReconnectLayer>,
    #[serde(default)]
    pub backoff: Option<BackoffLayer>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceTransportLayer {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub url: Option<url::Url>,
    #[serde(default)]
    pub headers: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default)]
    pub subject_prefix: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceLayer {
    #[serde(default, flatten)]
    pub transport: ServiceTransportLayer,
    #[serde(default)]
    pub service: Option<ServiceConfigLayer>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceConfigLayer {
    #[serde(default)]
    pub bind_addr: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub public_base_url: Option<String>,
    #[serde(default)]
    pub metrics: Option<MetricsLayer>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetricsLayer {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub bind_addr: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReconnectLayer {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub max_retries: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BackoffLayer {
    #[serde(default)]
    pub initial_ms: Option<u64>,
    #[serde(default)]
    pub max_ms: Option<u64>,
    #[serde(default)]
    pub multiplier: Option<f64>,
    #[serde(default)]
    pub jitter: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeLayer {
    #[serde(default)]
    pub max_concurrency: Option<u32>,
    #[serde(default)]
    pub task_timeout_ms: Option<u64>,
    #[serde(default)]
    pub shutdown_grace_ms: Option<u64>,
    #[serde(default)]
    pub admin_endpoints: Option<AdminEndpointsLayer>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AdminEndpointsLayer {
    #[serde(default)]
    pub secrets_explain_enabled: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TelemetryLayer {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub exporter: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub sampling: Option<f32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NetworkLayer {
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub tls_mode: Option<String>,
    #[serde(default)]
    pub connect_timeout_ms: Option<u64>,
    #[serde(default)]
    pub read_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeployerLayer {
    #[serde(default)]
    pub base_domain: Option<String>,
    #[serde(default)]
    pub provider: Option<DeployerProviderDefaultsLayer>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeployerProviderDefaultsLayer {
    #[serde(default)]
    pub provider_kind: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SecretsBackendRefLayer {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PacksLayer {
    #[serde(default)]
    pub source: Option<PackSourceLayer>,
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
    #[serde(default)]
    pub index_cache_ttl_secs: Option<u64>,
    #[serde(default)]
    pub trust: Option<PackTrustLayer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PackSourceLayer {
    LocalIndex {
        #[serde(default)]
        path: Option<PathBuf>,
    },
    HttpIndex {
        #[serde(default)]
        url: Option<String>,
    },
    OciRegistry {
        #[serde(default)]
        reference: Option<String>,
    },
}

impl Default for PackSourceLayer {
    fn default() -> Self {
        Self::LocalIndex { path: None }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PackTrustLayer {
    #[serde(default)]
    pub public_keys: Option<Vec<String>>,
    #[serde(default)]
    pub require_signatures: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DevLayer {
    #[serde(default)]
    pub default_env: Option<EnvId>,
    #[serde(default)]
    pub default_tenant: Option<String>,
    #[serde(default)]
    pub default_team: Option<String>,
}

pub fn default_layer(root: &Path, defaults: &DefaultPaths) -> ConfigLayer {
    ConfigLayer {
        schema_version: Some(ConfigVersion::v1()),
        environment: Some(EnvironmentLayer {
            env_id: Some(default_env_id()),
            deployment: None,
            connection: None,
            region: None,
        }),
        paths: Some(PathsLayer {
            greentic_root: Some(root.to_path_buf()),
            state_dir: Some(defaults.state_dir.clone()),
            cache_dir: Some(defaults.cache_dir.clone()),
            logs_dir: Some(defaults.logs_dir.clone()),
        }),
        services: None,
        events: Some(EventsLayer {
            reconnect: Some(ReconnectLayer {
                enabled: Some(true),
                max_retries: Some(50),
            }),
            backoff: Some(BackoffLayer {
                initial_ms: Some(250),
                max_ms: Some(30_000),
                multiplier: Some(2.0),
                jitter: Some(true),
            }),
        }),
        runtime: Some(RuntimeLayer::default()),
        telemetry: Some(TelemetryLayer {
            enabled: Some(true),
            exporter: Some("none".to_string()),
            endpoint: None,
            sampling: Some(1.0),
        }),
        network: Some(NetworkLayer {
            proxy_url: None,
            tls_mode: Some("system".to_string()),
            connect_timeout_ms: None,
            read_timeout_ms: None,
        }),
        deployer: Some(DeployerLayer {
            base_domain: Some(DEFAULT_DEPLOYER_BASE_DOMAIN.to_string()),
            provider: None,
        }),
        secrets: Some(SecretsBackendRefLayer {
            kind: Some("none".to_string()),
            reference: None,
        }),
        packs: None,
        dev: None,
    }
}

pub fn load_user_config() -> anyhow::Result<ConfigLayer> {
    let Some(dirs) = directories::ProjectDirs::from("com", "greentic", "greentic") else {
        return Ok(ConfigLayer::default());
    };
    let path = dirs.config_dir().join("config.toml");
    load_config_file_if_exists(&path)
}

pub fn load_user_config_with_origin() -> anyhow::Result<(ConfigLayer, Option<PathBuf>)> {
    let Some(dirs) = directories::ProjectDirs::from("com", "greentic", "greentic") else {
        return Ok((ConfigLayer::default(), None));
    };
    let path = dirs.config_dir().join("config.toml");
    let layer = load_config_file_if_exists(&path)?;
    let abs = crate::paths::absolute_path(&path)?;
    Ok((layer, Some(abs)))
}

pub fn load_project_config(project_root: &Path) -> anyhow::Result<ConfigLayer> {
    let path = project_root.join(".greentic").join("config.toml");
    load_config_file_if_exists(&path)
}

pub fn load_project_config_with_origin(
    project_root: &Path,
) -> anyhow::Result<(ConfigLayer, PathBuf)> {
    let path = project_root.join(".greentic").join("config.toml");
    let layer = load_project_config(project_root)?;
    let abs = crate::paths::absolute_path(&path)?;
    Ok((layer, abs))
}

pub fn load_config_file_required(path: &Path) -> anyhow::Result<ConfigLayer> {
    let abs = crate::paths::absolute_path(path)?;
    if !abs.exists() {
        return Err(anyhow::anyhow!(
            "explicit config file not found: {}\nHint: pass an existing file to --config / with_config_path()",
            abs.display()
        ));
    }
    let contents = fs::read_to_string(&abs)?;
    let format = match abs.extension().and_then(|s| s.to_str()) {
        Some("json") => ConfigFileFormat::Json,
        _ => ConfigFileFormat::Toml,
    };
    parse_layer(&contents, format)
}

fn load_config_file_if_exists(path: &Path) -> anyhow::Result<ConfigLayer> {
    if !path.exists() {
        return Ok(ConfigLayer::default());
    }
    let contents = fs::read_to_string(path)?;
    let format = match path.extension().and_then(|s| s.to_str()) {
        Some("json") => ConfigFileFormat::Json,
        _ => ConfigFileFormat::Toml,
    };
    let layer = parse_layer(&contents, format)?;
    Ok(layer)
}

fn parse_layer(contents: &str, format: ConfigFileFormat) -> anyhow::Result<ConfigLayer> {
    let layer = match format {
        ConfigFileFormat::Toml => toml::from_str::<ConfigLayer>(contents)?,
        ConfigFileFormat::Json => serde_json::from_str::<ConfigLayer>(contents)?,
    };
    Ok(layer)
}

pub fn load_env_layer() -> (ConfigLayer, Vec<String>) {
    let mut layer = ConfigLayer::default();
    let mut warnings = Vec::new();
    for (key, value) in std::env::vars() {
        if !key.starts_with(ENV_PREFIX) {
            continue;
        }
        apply_env_var(&key, &value, &mut layer, &mut warnings);
    }
    (layer, warnings)
}

pub fn load_env_layers_detailed() -> Vec<(ConfigLayer, String, Vec<String>)> {
    load_env_layers_detailed_from(std::env::vars())
}

pub fn load_env_layers_detailed_from<I>(vars: I) -> Vec<(ConfigLayer, String, Vec<String>)>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut layers = Vec::new();
    for (key, value) in vars {
        if !key.starts_with(ENV_PREFIX) {
            continue;
        }
        let mut layer = ConfigLayer::default();
        let mut warnings = Vec::new();
        if apply_env_var(&key, &value, &mut layer, &mut warnings) {
            layers.push((layer, key, warnings));
        }
    }
    layers
}

fn apply_env_var(
    key: &str,
    value: &str,
    layer: &mut ConfigLayer,
    warnings: &mut Vec<String>,
) -> bool {
    match key {
        "GREENTIC_SCHEMA_VERSION" => layer.schema_version = Some(ConfigVersion(value.to_string())),
        "GREENTIC_ENVIRONMENT_ENV_ID" => {
            layer
                .environment
                .get_or_insert_with(Default::default)
                .env_id = parse_string_as::<EnvId>(value)
        }
        "GREENTIC_ENVIRONMENT_DEPLOYMENT" => {
            layer
                .environment
                .get_or_insert_with(Default::default)
                .deployment = parse_string_as::<DeploymentCtx>(value)
        }
        "GREENTIC_ENVIRONMENT_CONNECTION" => {
            layer
                .environment
                .get_or_insert_with(Default::default)
                .connection = parse_string_as::<ConnectionKind>(value)
        }
        "GREENTIC_ENVIRONMENT_REGION" => {
            layer
                .environment
                .get_or_insert_with(Default::default)
                .region = Some(value.to_string())
        }
        "GREENTIC_PATHS_GREENTIC_ROOT" => {
            layer
                .paths
                .get_or_insert_with(Default::default)
                .greentic_root = Some(PathBuf::from(value))
        }
        "GREENTIC_PATHS_STATE_DIR" => {
            layer.paths.get_or_insert_with(Default::default).state_dir = Some(PathBuf::from(value))
        }
        "GREENTIC_PATHS_CACHE_DIR" => {
            layer.paths.get_or_insert_with(Default::default).cache_dir = Some(PathBuf::from(value))
        }
        "GREENTIC_PATHS_LOGS_DIR" => {
            layer.paths.get_or_insert_with(Default::default).logs_dir = Some(PathBuf::from(value))
        }
        "GREENTIC_SERVICES_EVENTS_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events
                .get_or_insert_with(Default::default)
                .url = parse_string_as::<url::Url>(value)
        }
        "GREENTIC_SERVICES_RUNNER_KIND" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .transport
                .kind = Some(value.to_lowercase())
        }
        "GREENTIC_SERVICES_RUNNER_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .transport
                .url = parse_string_as::<url::Url>(value)
        }
        "GREENTIC_SERVICES_RUNNER_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_RUNNER_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_RUNNER_PUBLIC_BASE_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .public_base_url = Some(value.to_string());
        }
        "GREENTIC_SERVICES_RUNNER_METRICS_ENABLED" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .enabled = parse_bool(value);
        }
        "GREENTIC_SERVICES_RUNNER_METRICS_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_RUNNER_METRICS_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_RUNNER_METRICS_PATH" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .runner
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .path = Some(value.to_string());
        }
        "GREENTIC_SERVICES_DEPLOYER_KIND" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .transport
                .kind = Some(value.to_lowercase())
        }
        "GREENTIC_SERVICES_DEPLOYER_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .transport
                .url = parse_string_as::<url::Url>(value)
        }
        "GREENTIC_SERVICES_DEPLOYER_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_DEPLOYER_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_DEPLOYER_PUBLIC_BASE_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .public_base_url = Some(value.to_string());
        }
        "GREENTIC_SERVICES_DEPLOYER_METRICS_ENABLED" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .enabled = parse_bool(value);
        }
        "GREENTIC_SERVICES_DEPLOYER_METRICS_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_DEPLOYER_METRICS_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_DEPLOYER_METRICS_PATH" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .deployer
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .path = Some(value.to_string());
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_KIND" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .transport
                .kind = Some(value.to_lowercase())
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .transport
                .url = parse_string_as::<url::Url>(value)
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_PUBLIC_BASE_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .public_base_url = Some(value.to_string());
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_METRICS_ENABLED" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .enabled = parse_bool(value);
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_METRICS_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_METRICS_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_EVENTS_TRANSPORT_METRICS_PATH" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .events_transport
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .path = Some(value.to_string());
        }
        "GREENTIC_SERVICES_SOURCE_KIND" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .transport
                .kind = Some(value.to_lowercase())
        }
        "GREENTIC_SERVICES_SOURCE_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .transport
                .url = parse_string_as::<url::Url>(value)
        }
        "GREENTIC_SERVICES_SOURCE_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_SOURCE_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_SOURCE_PUBLIC_BASE_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .public_base_url = Some(value.to_string());
        }
        "GREENTIC_SERVICES_SOURCE_METRICS_ENABLED" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .enabled = parse_bool(value);
        }
        "GREENTIC_SERVICES_SOURCE_METRICS_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_SOURCE_METRICS_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_SOURCE_METRICS_PATH" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .source
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .path = Some(value.to_string());
        }
        "GREENTIC_SERVICES_PUBLISH_KIND" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .transport
                .kind = Some(value.to_lowercase())
        }
        "GREENTIC_SERVICES_PUBLISH_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .transport
                .url = parse_string_as::<url::Url>(value)
        }
        "GREENTIC_SERVICES_PUBLISH_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_PUBLISH_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_PUBLISH_PUBLIC_BASE_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .public_base_url = Some(value.to_string());
        }
        "GREENTIC_SERVICES_PUBLISH_METRICS_ENABLED" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .enabled = parse_bool(value);
        }
        "GREENTIC_SERVICES_PUBLISH_METRICS_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_PUBLISH_METRICS_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_PUBLISH_METRICS_PATH" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .publish
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .path = Some(value.to_string());
        }
        "GREENTIC_SERVICES_METADATA_KIND" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .transport
                .kind = Some(value.to_lowercase())
        }
        "GREENTIC_SERVICES_METADATA_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .transport
                .url = parse_string_as::<url::Url>(value)
        }
        "GREENTIC_SERVICES_METADATA_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_METADATA_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_METADATA_PUBLIC_BASE_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .public_base_url = Some(value.to_string());
        }
        "GREENTIC_SERVICES_METADATA_METRICS_ENABLED" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .enabled = parse_bool(value);
        }
        "GREENTIC_SERVICES_METADATA_METRICS_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_METADATA_METRICS_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_METADATA_METRICS_PATH" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .metadata
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .path = Some(value.to_string());
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_KIND" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .transport
                .kind = Some(value.to_lowercase())
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .transport
                .url = parse_string_as::<url::Url>(value)
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_PUBLIC_BASE_URL" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .public_base_url = Some(value.to_string());
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_METRICS_ENABLED" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .enabled = parse_bool(value);
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_METRICS_BIND_ADDR" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .bind_addr = Some(value.to_string());
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_METRICS_PORT" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .port = parse_u16_warn(value, key, warnings);
        }
        "GREENTIC_SERVICES_OAUTH_BROKER_METRICS_PATH" => {
            layer
                .services
                .get_or_insert_with(Default::default)
                .oauth_broker
                .get_or_insert_with(Default::default)
                .service
                .get_or_insert_with(Default::default)
                .metrics
                .get_or_insert_with(Default::default)
                .path = Some(value.to_string());
        }
        "GREENTIC_RUNTIME_MAX_CONCURRENCY" => {
            layer
                .runtime
                .get_or_insert_with(Default::default)
                .max_concurrency = parse_u32(value)
        }
        "GREENTIC_RUNTIME_TASK_TIMEOUT_MS" => {
            layer
                .runtime
                .get_or_insert_with(Default::default)
                .task_timeout_ms = parse_u64(value)
        }
        "GREENTIC_RUNTIME_SHUTDOWN_GRACE_MS" => {
            layer
                .runtime
                .get_or_insert_with(Default::default)
                .shutdown_grace_ms = parse_u64(value)
        }
        "GREENTIC_RUNTIME_ADMIN_SECRETS_EXPLAIN_ENABLED" => {
            layer
                .runtime
                .get_or_insert_with(Default::default)
                .admin_endpoints
                .get_or_insert_with(Default::default)
                .secrets_explain_enabled = parse_bool(value)
        }
        "GREENTIC_TELEMETRY_ENABLED" => {
            layer.telemetry.get_or_insert_with(Default::default).enabled = parse_bool(value)
        }
        "GREENTIC_TELEMETRY_EXPORTER" => {
            layer
                .telemetry
                .get_or_insert_with(Default::default)
                .exporter = Some(value.to_lowercase())
        }
        "GREENTIC_TELEMETRY_ENDPOINT" => {
            layer
                .telemetry
                .get_or_insert_with(Default::default)
                .endpoint = Some(value.to_string())
        }
        "GREENTIC_TELEMETRY_SAMPLING" => {
            layer
                .telemetry
                .get_or_insert_with(Default::default)
                .sampling = parse_f32(value)
        }
        "GREENTIC_NETWORK_PROXY_URL" => {
            layer.network.get_or_insert_with(Default::default).proxy_url = Some(value.to_string())
        }
        "GREENTIC_NETWORK_TLS_MODE" => {
            layer.network.get_or_insert_with(Default::default).tls_mode = Some(value.to_lowercase())
        }
        "GREENTIC_NETWORK_CONNECT_TIMEOUT_MS" => {
            layer
                .network
                .get_or_insert_with(Default::default)
                .connect_timeout_ms = parse_u64(value)
        }
        "GREENTIC_NETWORK_READ_TIMEOUT_MS" => {
            layer
                .network
                .get_or_insert_with(Default::default)
                .read_timeout_ms = parse_u64(value)
        }
        "GREENTIC_SECRETS_KIND" => {
            layer.secrets.get_or_insert_with(Default::default).kind = Some(value.to_string())
        }
        "GREENTIC_SECRETS_REFERENCE" => {
            layer.secrets.get_or_insert_with(Default::default).reference = Some(value.to_string())
        }
        "GREENTIC_DEV_DEFAULT_ENV" => {
            layer.dev.get_or_insert_with(Default::default).default_env =
                parse_string_as::<EnvId>(value)
        }
        "GREENTIC_DEV_DEFAULT_TENANT" => {
            layer
                .dev
                .get_or_insert_with(Default::default)
                .default_tenant = Some(value.to_string())
        }
        "GREENTIC_DEV_DEFAULT_TEAM" => {
            layer.dev.get_or_insert_with(Default::default).default_team = Some(value.to_string())
        }
        "GREENTIC_EVENTS_RECONNECT_ENABLED" => {
            layer
                .events
                .get_or_insert_with(Default::default)
                .reconnect
                .get_or_insert_with(Default::default)
                .enabled = parse_bool(value)
        }
        "GREENTIC_EVENTS_RECONNECT_MAX_RETRIES" => {
            layer
                .events
                .get_or_insert_with(Default::default)
                .reconnect
                .get_or_insert_with(Default::default)
                .max_retries = parse_u32(value)
        }
        "GREENTIC_EVENTS_BACKOFF_INITIAL_MS" => {
            layer
                .events
                .get_or_insert_with(Default::default)
                .backoff
                .get_or_insert_with(Default::default)
                .initial_ms = parse_u64(value)
        }
        "GREENTIC_EVENTS_BACKOFF_MAX_MS" => {
            layer
                .events
                .get_or_insert_with(Default::default)
                .backoff
                .get_or_insert_with(Default::default)
                .max_ms = parse_u64(value)
        }
        "GREENTIC_EVENTS_BACKOFF_MULTIPLIER" => {
            layer
                .events
                .get_or_insert_with(Default::default)
                .backoff
                .get_or_insert_with(Default::default)
                .multiplier = parse_f32(value).map(|v| v as f64)
        }
        "GREENTIC_EVENTS_BACKOFF_JITTER" => {
            layer
                .events
                .get_or_insert_with(Default::default)
                .backoff
                .get_or_insert_with(Default::default)
                .jitter = parse_bool(value)
        }
        _ => return false,
    }
    true
}

fn parse_string_as<T: for<'de> Deserialize<'de>>(value: &str) -> Option<T> {
    serde_json::from_str(&format!("\"{value}\"")).ok()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_u64(value: &str) -> Option<u64> {
    value.parse::<u64>().ok()
}

fn parse_u32(value: &str) -> Option<u32> {
    value.parse::<u32>().ok()
}

fn parse_f32(value: &str) -> Option<f32> {
    value.parse::<f32>().ok()
}

fn parse_u16_warn(value: &str, key: &str, warnings: &mut Vec<String>) -> Option<u16> {
    match value.parse::<u16>() {
        Ok(v) => Some(v),
        Err(_) => {
            warnings.push(format!("Ignored {key}: expected u16 but got '{value}'"));
            None
        }
    }
}

pub(crate) fn default_env_id() -> EnvId {
    serde_json::from_str("\"dev\"").expect("EnvId should deserialize from string")
}
