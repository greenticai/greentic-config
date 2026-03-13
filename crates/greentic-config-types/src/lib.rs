use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use url::Url;

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
    #[serde(default)]
    pub services: Option<ServicesConfig>,
    #[serde(default)]
    pub events: Option<EventsConfig>,
    pub runtime: RuntimeConfig,
    pub telemetry: TelemetryConfig,
    pub network: NetworkConfig,
    #[serde(default)]
    pub deployer: Option<DeployerConfig>,
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

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServicesConfig {
    #[serde(default)]
    pub events: Option<ServiceEndpointConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner: Option<ServiceDefinitionConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployer: Option<ServiceDefinitionConfig>,

    /// Transport selector for events.
    ///
    /// This exists alongside `services.events` for backward compatibility:
    /// - `services.events` preserves the legacy HTTP-only endpoint shape.
    /// - New consumers should prefer `services.events_transport`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events_transport: Option<ServiceDefinitionConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ServiceDefinitionConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish: Option<ServiceDefinitionConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<ServiceDefinitionConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ServiceDefinitionConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_broker: Option<ServiceDefinitionConfig>,
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceEndpointConfig {
    pub url: Url,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
}

// --- Services transport (non-secret) ---

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServiceDefinitionConfig {
    pub transport: Option<ServiceTransportConfig>,
    pub service: Option<ServiceConfig>,
}

impl Serialize for ServiceDefinitionConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.service.is_none() && self.transport.is_some() {
            return self
                .transport
                .as_ref()
                .expect("checked is_some")
                .serialize(serializer);
        }

        let mut map = serde_json::Map::new();
        if let Some(transport) = &self.transport {
            let value = serde_json::to_value(transport).map_err(serde::ser::Error::custom)?;
            let obj = value
                .as_object()
                .ok_or_else(|| serde::ser::Error::custom("expected map for transport"))?;
            for (k, v) in obj {
                map.insert(k.clone(), v.clone());
            }
        }
        if let Some(service) = &self.service {
            map.insert(
                "service".to_string(),
                serde_json::to_value(service).map_err(serde::ser::Error::custom)?,
            );
        }
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ServiceDefinitionConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Object(mut map) = value {
            let service = if let Some(service_value) = map.remove("service") {
                Some(ServiceConfig::deserialize(service_value).map_err(serde::de::Error::custom)?)
            } else {
                None
            };

            let transport = if map.is_empty() {
                None
            } else {
                let transport_value = serde_json::Value::Object(map);
                ServiceTransportConfig::deserialize(transport_value).ok()
            };

            return Ok(ServiceDefinitionConfig { transport, service });
        }

        // Fallback to transport-only shape
        let transport = ServiceTransportConfig::deserialize(value).ok();

        Ok(ServiceDefinitionConfig {
            transport,
            service: None,
        })
    }
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ServiceTransportConfig {
    /// Explicitly disables this service integration.
    Noop,

    /// HTTP transport with a base URL and optional headers.
    ///
    /// Headers are strictly for non-sensitive routing/metadata only and MUST NOT include secrets
    /// (e.g. `Authorization`, `Cookie`, `Set-Cookie`).
    Http {
        url: url::Url,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        headers: Option<std::collections::BTreeMap<String, String>>,
    },

    /// NATS transport (non-secret). Auth is handled elsewhere (secrets-store), not here.
    Nats {
        url: url::Url,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subject_prefix: Option<String>,
    },
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
    #[serde(default)]
    pub bind_addr: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub public_base_url: Option<String>,
    #[serde(default)]
    pub metrics: Option<MetricsConfig>,
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsConfig {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub bind_addr: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventsConfig {
    #[serde(default)]
    pub reconnect: Option<ReconnectConfig>,
    #[serde(default)]
    pub backoff: Option<BackoffConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReconnectConfig {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub max_retries: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackoffConfig {
    #[serde(default)]
    pub initial_ms: Option<u64>,
    #[serde(default)]
    pub max_ms: Option<u64>,
    #[serde(default)]
    pub multiplier: Option<f64>,
    #[serde(default)]
    pub jitter: Option<bool>,
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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_endpoints: Option<AdminEndpointsConfig>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminEndpointsConfig {
    /// Enables sensitive admin endpoints (default: false).
    #[serde(default)]
    pub secrets_explain_enabled: bool,
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
    Gcp,
    Azure,
    Aws,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeployerConfig {
    /// Default domain used when generating deployment URLs / routing domains.
    #[serde(default)]
    pub base_domain: Option<String>,
    #[serde(default)]
    pub provider: Option<DeployerProviderDefaults>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeployerProviderDefaults {
    #[serde(default)]
    pub provider_kind: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
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
    use url::Url;

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

[services.runner]
kind = "http"
url = "https://runner.greentic.local"
headers = { "x-routing-key" = "tenant-1" }

[services.runner.service]
bind_addr = "0.0.0.0"
port = 8080
public_base_url = "https://runner-public.greentic.local"

[services.runner.service.metrics]
enabled = true
bind_addr = "127.0.0.1"
port = 9090
path = "/metrics"

[services.deployer]
kind = "nats"
url = "nats://nats.greentic.local:4222"
subject_prefix = "greentic"

[services.metadata]
kind = "noop"

[services.events]
url = "https://events.greentic.local"
headers = { "x-routing-key" = "tenant-1" }

[events.reconnect]
enabled = true
max_retries = 25

[events.backoff]
initial_ms = 250
max_ms = 30000
multiplier = 2.0
jitter = true

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

[runtime.admin_endpoints]
secrets_explain_enabled = true

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

[deployer]
base_domain = "deploy.greentic.ai"

[deployer.provider]
provider_kind = "aws"
region = "us-west-2"

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

        let runner = config_back
            .services
            .as_ref()
            .and_then(|s| s.runner.as_ref())
            .expect("runner present");
        let service = runner.service.as_ref().expect("service binding present");
        assert_eq!(service.bind_addr.as_deref(), Some("0.0.0.0"));
        assert_eq!(service.port, Some(8080));
        assert_eq!(
            service.public_base_url.as_deref(),
            Some("https://runner-public.greentic.local")
        );
        let metrics = service.metrics.as_ref().expect("metrics present");
        assert_eq!(metrics.enabled, Some(true));
        assert_eq!(metrics.bind_addr.as_deref(), Some("127.0.0.1"));
        assert_eq!(metrics.port, Some(9090));
        assert_eq!(metrics.path.as_deref(), Some("/metrics"));
        assert!(
            config_back
                .services
                .as_ref()
                .and_then(|s| s.store.as_ref())
                .is_none(),
            "store should remain absent when omitted"
        );
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
            "services": {
                "events": {
                    "url": "https://events.greentic.local",
                    "headers": {"x-routing-key": "tenant-1"}
                },
                "runner": {
                    "kind": "http",
                    "url": "https://runner.greentic.local",
                    "headers": {"x-routing-key": "tenant-1"},
                    "service": {
                        "bind_addr": "0.0.0.0",
                        "port": 8080,
                        "public_base_url": "https://runner-public.greentic.local",
                        "metrics": {
                            "enabled": true,
                            "bind_addr": "127.0.0.1",
                            "port": 9090,
                            "path": "/metrics"
                        }
                    }
                },
                "deployer": {"kind": "nats", "url": "nats://nats.greentic.local:4222", "subject_prefix": "greentic"},
                "store": {
                    "kind": "http",
                    "url": "https://store.greentic.local",
                    "service": {
                        "bind_addr": "0.0.0.0",
                        "port": 7070,
                        "metrics": {"enabled": true, "port": 9191}
                    }
                },
                "metadata": {"kind": "noop"}
            },
            "events": {
                "reconnect": {"enabled": true, "max_retries": 10},
                "backoff": {"initial_ms": 100, "max_ms": 5000, "multiplier": 1.5, "jitter": false}
            },
            "packs": {
                "cache_dir": "/workspace/.greentic/cache/packs",
                "index_cache_ttl_secs": 3600,
                "source": {"type": "http_index", "url": "https://example.com/index.json"},
                "trust": {"public_keys": ["inline-key", "/keys/key.pem"], "require_signatures": true}
            },
            "runtime": {"max_concurrency": 4, "task_timeout_ms": 120000, "shutdown_grace_ms": 1000, "admin_endpoints": {"secrets_explain_enabled": true}},
            "telemetry": {"enabled": true, "exporter": "stdout", "sampling": 1.0},
            "network": {"tls_mode": "system"},
            "deployer": {"base_domain": "deploy.greentic.ai", "provider": {"provider_kind": "gcp", "region": "europe-west1"}},
            "secrets": {"kind": "none"},
            "dev": {"default_env": "dev", "default_tenant": "acme"}
        }"#,
        )
        .expect("json decode");

        let serialized = serde_json::to_string(&config).expect("json encode");
        let round: GreenticConfig = serde_json::from_str(&serialized).expect("json decode round");
        assert_eq!(config, round);

        let runner = round
            .services
            .as_ref()
            .and_then(|s| s.runner.as_ref())
            .expect("runner present");
        let service = runner.service.as_ref().expect("service binding present");
        assert_eq!(service.port, Some(8080));

        let store = round
            .services
            .as_ref()
            .and_then(|s| s.store.as_ref())
            .expect("store present");
        let store_transport = store.transport.as_ref().expect("store transport present");
        match store_transport {
            ServiceTransportConfig::Http { url, .. } => {
                assert_eq!(url.as_str(), "https://store.greentic.local/")
            }
            other => panic!("unexpected transport {other:?}"),
        }
        let store_service = store.service.as_ref().expect("store binding present");
        assert_eq!(store_service.port, Some(7070));
        let store_metrics = store_service
            .metrics
            .as_ref()
            .expect("store metrics present");
        assert_eq!(store_metrics.port, Some(9191));
    }

    #[test]
    fn store_service_round_trips_with_bindings() {
        let services: ServicesConfig = toml::from_str(
            r#"
[events]
url = "https://events.greentic.local"

[store]
kind = "http"
url = "https://store.greentic.local"

[store.service]
bind_addr = "0.0.0.0"
port = 7070
public_base_url = "https://store-public.greentic.local"

[store.service.metrics]
enabled = true
bind_addr = "127.0.0.1"
port = 9191
path = "/metrics"
"#,
        )
        .expect("deserialize store service");

        let serialized = toml::to_string(&services).expect("serialize store");
        let round: ServicesConfig = toml::from_str(&serialized).expect("deserialize store");

        let store = round.store.expect("store config present");
        let transport = store.transport.expect("transport present");
        match transport {
            ServiceTransportConfig::Http { url, .. } => {
                assert_eq!(url.as_str(), "https://store.greentic.local/")
            }
            other => panic!("unexpected transport {other:?}"),
        }

        let service = store.service.expect("service present");
        assert_eq!(service.bind_addr.as_deref(), Some("0.0.0.0"));
        assert_eq!(service.port, Some(7070));
        assert_eq!(
            service.public_base_url.as_deref(),
            Some("https://store-public.greentic.local")
        );

        let metrics = service.metrics.expect("metrics present");
        assert_eq!(metrics.enabled, Some(true));
        assert_eq!(metrics.bind_addr.as_deref(), Some("127.0.0.1"));
        assert_eq!(metrics.port, Some(9191));
        assert_eq!(metrics.path.as_deref(), Some("/metrics"));
    }

    #[test]
    fn service_definition_accepts_transport_only_shape() {
        let services: ServicesConfig = toml::from_str(
            r#"
[runner]
kind = "http"
url = "https://runner.greentic.local"
            "#,
        )
        .expect("deserialize runner");

        let runner = services.runner.expect("runner");
        assert!(runner.service.is_none());
        let transport = runner.transport.expect("transport");
        match transport {
            ServiceTransportConfig::Http { url, .. } => {
                assert_eq!(url.as_str(), "https://runner.greentic.local/")
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn services_and_events_are_schema_only() {
        let endpoint = ServiceEndpointConfig {
            url: Url::parse("https://events.greentic.local").unwrap(),
            headers: None,
        };
        let services = ServicesConfig {
            events: Some(endpoint),
            ..Default::default()
        };
        let events = EventsConfig {
            reconnect: Some(ReconnectConfig {
                enabled: Some(true),
                max_retries: Some(5),
            }),
            backoff: Some(BackoffConfig {
                initial_ms: Some(100),
                max_ms: Some(1000),
                multiplier: Some(2.0),
                jitter: Some(true),
            }),
        };

        let serialized = toml::to_string(&services).expect("serialize services");
        let services_back: ServicesConfig =
            toml::from_str(&serialized).expect("deserialize services");
        assert_eq!(
            services_back.events.unwrap().url.as_str(),
            "https://events.greentic.local/"
        );

        let serialized_events = serde_json::to_string(&events).expect("serialize events");
        let events_back: EventsConfig =
            serde_json::from_str(&serialized_events).expect("deserialize events");
        assert_eq!(events_back.backoff.unwrap().initial_ms, Some(100));
    }

    #[test]
    fn backward_compat_services_events_still_deserializes() {
        let legacy_services: ServicesConfig = toml::from_str(
            r#"
[events]
url = "https://events.greentic.local"
headers = { "x-routing-key" = "tenant-1" }
"#,
        )
        .expect("deserialize legacy services.events shape");
        assert_eq!(
            legacy_services.events.unwrap().url.as_str(),
            "https://events.greentic.local/"
        );
        assert!(legacy_services.events_transport.is_none());
        assert!(legacy_services.runner.is_none());
        assert!(legacy_services.deployer.is_none());
        assert!(legacy_services.metadata.is_none());
    }

    #[test]
    fn service_transport_config_serializes_with_kind_tags() {
        let noop = ServiceTransportConfig::Noop;
        let http = ServiceTransportConfig::Http {
            url: Url::parse("https://runner.greentic.local").unwrap(),
            headers: None,
        };
        let nats = ServiceTransportConfig::Nats {
            url: Url::parse("nats://nats.greentic.local:4222").unwrap(),
            subject_prefix: Some("greentic".to_string()),
        };

        let noop_v = serde_json::to_value(&noop).expect("json");
        assert_eq!(noop_v, serde_json::json!({"kind": "noop"}));

        let http_v = serde_json::to_value(&http).expect("json");
        assert_eq!(
            http_v,
            serde_json::json!({"kind": "http", "url": "https://runner.greentic.local/"})
        );

        let nats_v = serde_json::to_value(&nats).expect("json");
        assert_eq!(
            nats_v,
            serde_json::json!({"kind": "nats", "url": "nats://nats.greentic.local:4222", "subject_prefix": "greentic"})
        );
    }
}
