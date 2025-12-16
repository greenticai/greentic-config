use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use greentic_types::{ConnectionKind, DeploymentCtx, EnvId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigVersion(pub String);

impl ConfigVersion {
    pub fn v1() -> Self {
        Self("1".to_string())
    }
}

impl Default for ConfigVersion {
    fn default() -> Self {
        Self::v1()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GreenticConfig {
    #[serde(default = "ConfigVersion::v1")]
    pub schema_version: ConfigVersion,
    pub environment: EnvironmentConfig,
    pub paths: PathsConfig,
    #[serde(default)]
    pub packs: Option<PacksConfig>,
    pub runtime: RuntimeConfig,
    pub telemetry: TelemetryConfig,
    pub network: NetworkConfig,
    pub secrets: SecretsBackendRefConfig,
    pub dev: Option<DevConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigSource {
    Default,
    User,
    Project,
    Environment,
    Cli,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProvenancePath(pub String);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub env_id: EnvId,
    pub deployment: Option<DeploymentCtx>,
    pub connection: Option<ConnectionKind>,
    pub region: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathsConfig {
    pub greentic_root: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub logs_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacksConfig {
    pub source: PackSourceConfig,
    pub cache_dir: PathBuf,
    #[serde(default)]
    pub index_cache_ttl_secs: Option<u64>,
    #[serde(default)]
    pub trust: Option<PackTrustConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PackSourceConfig {
    LocalIndex { path: PathBuf },
    HttpIndex { url: String },
    OciRegistry { reference: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackTrustConfig {
    #[serde(default)]
    pub public_keys: Vec<String>,
    #[serde(default)]
    pub require_signatures: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub max_concurrency: Option<u32>,
    #[serde(default)]
    pub task_timeout_ms: Option<u64>,
    #[serde(default)]
    pub shutdown_grace_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub exporter: TelemetryExporterKind,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default = "default_sampling")]
    pub sampling: f32,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            exporter: TelemetryExporterKind::None,
            endpoint: None,
            sampling: default_sampling(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_sampling() -> f32 {
    1.0
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryExporterKind {
    Otlp,
    Stdout,
    #[default]
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub tls_mode: TlsMode,
    #[serde(default)]
    pub connect_timeout_ms: Option<u64>,
    #[serde(default)]
    pub read_timeout_ms: Option<u64>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy_url: None,
            tls_mode: TlsMode::System,
            connect_timeout_ms: None,
            read_timeout_ms: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TlsMode {
    Disabled,
    #[default]
    System,
    Strict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretsBackendRefConfig {
    #[serde(default = "default_backend_kind")]
    pub kind: String,
    #[serde(default)]
    pub reference: Option<String>,
}

impl Default for SecretsBackendRefConfig {
    fn default() -> Self {
        Self {
            kind: default_backend_kind(),
            reference: None,
        }
    }
}

fn default_backend_kind() -> String {
    "none".to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevConfig {
    pub default_env: EnvId,
    pub default_tenant: String,
    #[serde(default)]
    pub default_team: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_toml() -> &'static str {
        r#"
schema_version = "1"

[environment]
env_id = "dev"
region = "us-east-1"

[paths]
greentic_root = "/workspace"
state_dir = "/workspace/.greentic"
cache_dir = "/workspace/.greentic/cache"
logs_dir = "/workspace/.greentic/logs"

[packs]
cache_dir = "/workspace/.greentic/cache/packs"
index_cache_ttl_secs = 3600

[packs.source]
type = "local_index"
path = "/workspace/.greentic/packs/index.json"

[packs.trust]
public_keys = ["/keys/key1.pem", "/keys/key2.pem"]
require_signatures = true

[runtime]
max_concurrency = 8
task_timeout_ms = 30000
shutdown_grace_ms = 5000

[telemetry]
enabled = true
exporter = "otlp"
endpoint = "http://localhost:4317"
sampling = 0.5

[network]
proxy_url = "http://proxy"
tls_mode = "system"
connect_timeout_ms = 1000
read_timeout_ms = 2000

[secrets]
kind = "vault"
reference = "ops"

[dev]
default_env = "dev"
default_tenant = "acme"
default_team = "devex"
"#
    }

    #[test]
    fn toml_round_trip() {
        let config: GreenticConfig = toml::from_str(sample_toml()).expect("deserialize");
        let serialized = toml::to_string(&config).expect("serialize");
        let config_back: GreenticConfig = toml::from_str(&serialized).expect("deserialize again");
        assert_eq!(config, config_back);
    }

    #[test]
    fn json_round_trip() {
        let config: GreenticConfig = serde_json::from_str(
            r#"{
            "schema_version": "1",
            "environment": {"env_id": "dev", "region": "us-west-2"},
            "paths": {
                "greentic_root": "/workspace",
                "state_dir": "/workspace/.greentic",
                "cache_dir": "/workspace/.greentic/cache",
                "logs_dir": "/workspace/.greentic/logs"
            },
            "packs": {
                "cache_dir": "/workspace/.greentic/cache/packs",
                "index_cache_ttl_secs": 3600,
                "source": {"type": "http_index", "url": "https://example.com/index.json"},
                "trust": {"public_keys": ["inline-key", "/keys/key.pem"], "require_signatures": true}
            },
            "runtime": {"max_concurrency": 4, "task_timeout_ms": 120000, "shutdown_grace_ms": 1000},
            "telemetry": {"enabled": true, "exporter": "stdout", "sampling": 1.0},
            "network": {"tls_mode": "system"},
            "secrets": {"kind": "none"},
            "dev": {"default_env": "dev", "default_tenant": "acme"}
        }"#,
        )
        .expect("json decode");

        let serialized = serde_json::to_string(&config).expect("json encode");
        let round: GreenticConfig = serde_json::from_str(&serialized).expect("json decode round");
        assert_eq!(config, round);
    }
}
