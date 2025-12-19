//! Enterprise-ready configuration resolver for Greentic hosts.
//!
//! This crate loads `GreenticConfig` (from `greentic-config-types`) from defaults, user config,
//! project config, environment variables, and CLI overrides with strict precedence:
//! `CLI > env > project > user > defaults`.
//!
//! Use `ConfigResolver::load()` for source-only provenance, or `ConfigResolver::load_detailed()` for
//! per-leaf provenance that also includes origin (file path / env var name / `cli`).

mod explain;
mod loaders;
mod merge;
mod paths;
mod validate;

use greentic_config_types::{ConfigSource, GreenticConfig, ProvenancePath};
use std::collections::HashMap;
use std::path::PathBuf;

pub use explain::{ExplainReport, explain, explain_detailed};
pub use loaders::{ConfigFileFormat, ConfigLayer};
pub use paths::{DefaultPaths, discover_project_root};
pub use validate::{ValidationError, validate_config, validate_config_with_overrides};

pub type ProvenanceMap = HashMap<ProvenancePath, ConfigSource>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceRecord {
    pub source: ConfigSource,
    pub origin: Option<String>,
}

pub type ProvenanceMapDetailed = HashMap<ProvenancePath, ProvenanceRecord>;

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub config: GreenticConfig,
    pub provenance: ProvenanceMap,
    pub warnings: Vec<String>,
}

impl ResolvedConfig {
    pub fn explain(&self) -> ExplainReport {
        explain(&self.config, &self.provenance, &self.warnings)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConfigDetailed {
    pub config: GreenticConfig,
    pub provenance: ProvenanceMapDetailed,
    pub warnings: Vec<String>,
}

impl ResolvedConfigDetailed {
    pub fn explain(&self) -> ExplainReport {
        explain::explain_detailed(&self.config, &self.provenance, &self.warnings)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    layer: ConfigLayer,
}

impl CliOverrides {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_env_id(mut self, env_id: greentic_config_types::EnvId) -> Self {
        self.layer
            .environment
            .get_or_insert_with(Default::default)
            .env_id = Some(env_id);
        self
    }

    pub fn with_connection(mut self, connection: greentic_config_types::ConnectionKind) -> Self {
        self.layer
            .environment
            .get_or_insert_with(Default::default)
            .connection = Some(connection);
        self
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.layer
            .environment
            .get_or_insert_with(Default::default)
            .region = Some(region.into());
        self
    }

    pub fn with_services_events_url(mut self, url: url::Url) -> Self {
        self.layer
            .services
            .get_or_insert_with(Default::default)
            .events
            .get_or_insert_with(Default::default)
            .url = Some(url);
        self
    }

    pub fn with_services_runner_transport(
        mut self,
        transport: greentic_config_types::ServiceTransportConfig,
    ) -> Self {
        self.layer
            .services
            .get_or_insert_with(Default::default)
            .runner = Some(transport_to_layer(transport));
        self
    }

    pub fn with_services_deployer_transport(
        mut self,
        transport: greentic_config_types::ServiceTransportConfig,
    ) -> Self {
        self.layer
            .services
            .get_or_insert_with(Default::default)
            .deployer = Some(transport_to_layer(transport));
        self
    }

    pub fn with_services_events_transport(
        mut self,
        transport: greentic_config_types::ServiceTransportConfig,
    ) -> Self {
        self.layer
            .services
            .get_or_insert_with(Default::default)
            .events_transport = Some(transport_to_layer(transport));
        self
    }

    pub fn with_services_source_transport(
        mut self,
        transport: greentic_config_types::ServiceTransportConfig,
    ) -> Self {
        self.layer
            .services
            .get_or_insert_with(Default::default)
            .source = Some(transport_to_layer(transport));
        self
    }

    pub fn with_services_publish_transport(
        mut self,
        transport: greentic_config_types::ServiceTransportConfig,
    ) -> Self {
        self.layer
            .services
            .get_or_insert_with(Default::default)
            .publish = Some(transport_to_layer(transport));
        self
    }

    pub fn with_services_metadata_transport(
        mut self,
        transport: greentic_config_types::ServiceTransportConfig,
    ) -> Self {
        self.layer
            .services
            .get_or_insert_with(Default::default)
            .metadata = Some(transport_to_layer(transport));
        self
    }

    pub fn with_services_oauth_broker_transport(
        mut self,
        transport: greentic_config_types::ServiceTransportConfig,
    ) -> Self {
        self.layer
            .services
            .get_or_insert_with(Default::default)
            .oauth_broker = Some(transport_to_layer(transport));
        self
    }

    pub fn with_runtime_admin_secrets_explain_enabled(mut self, enabled: bool) -> Self {
        self.layer
            .runtime
            .get_or_insert_with(Default::default)
            .admin_endpoints
            .get_or_insert_with(Default::default)
            .secrets_explain_enabled = Some(enabled);
        self
    }

    pub fn into_layer(self) -> ConfigLayer {
        self.layer
    }
}

impl From<CliOverrides> for ConfigLayer {
    fn from(value: CliOverrides) -> Self {
        value.layer
    }
}

fn transport_to_layer(
    transport: greentic_config_types::ServiceTransportConfig,
) -> crate::loaders::ServiceTransportLayer {
    match transport {
        greentic_config_types::ServiceTransportConfig::Noop => {
            crate::loaders::ServiceTransportLayer {
                kind: Some("noop".into()),
                ..Default::default()
            }
        }
        greentic_config_types::ServiceTransportConfig::Http { url, headers } => {
            crate::loaders::ServiceTransportLayer {
                kind: Some("http".into()),
                url: Some(url),
                headers,
                ..Default::default()
            }
        }
        greentic_config_types::ServiceTransportConfig::Nats {
            url,
            subject_prefix,
        } => crate::loaders::ServiceTransportLayer {
            kind: Some("nats".into()),
            url: Some(url),
            subject_prefix,
            ..Default::default()
        },
    }
}

#[derive(Debug, Clone)]
pub struct ConfigResolver {
    project_root: Option<PathBuf>,
    config_path: Option<PathBuf>,
    cli_overrides: Option<ConfigLayer>,
    allow_dev: bool,
    allow_network: bool,
}

impl ConfigResolver {
    pub fn new() -> Self {
        Self {
            project_root: None,
            config_path: None,
            cli_overrides: None,
            allow_dev: false,
            allow_network: false,
        }
    }

    pub fn with_project_root(mut self, root: PathBuf) -> Self {
        self.project_root = Some(root);
        self
    }

    pub fn with_project_root_opt(mut self, root: Option<PathBuf>) -> Self {
        if let Some(r) = root {
            self.project_root = Some(r);
        }
        self
    }

    pub fn with_config_path(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }

    pub fn with_cli_overrides(mut self, layer: ConfigLayer) -> Self {
        self.cli_overrides = Some(layer);
        self
    }

    pub fn with_cli_overrides_typed(mut self, overrides: CliOverrides) -> Self {
        self.cli_overrides = Some(overrides.into_layer());
        self
    }

    pub fn with_allow_dev(mut self, allow: bool) -> Self {
        self.allow_dev = allow;
        self
    }

    pub fn allow_dev(self, allow: bool) -> Self {
        self.with_allow_dev(allow)
    }

    pub fn with_allow_network(mut self, allow: bool) -> Self {
        self.allow_network = allow;
        self
    }

    pub fn load(&self) -> anyhow::Result<ResolvedConfig> {
        let project_root = self.resolve_project_root()?;

        let default_paths = DefaultPaths::from_root(&project_root);
        let mut provenance = ProvenanceMap::new();

        let default_layer = loaders::default_layer(&project_root, &default_paths);
        let user_layer = loaders::load_user_config()?;
        let (project_layer, _) = self.load_project_layer(&project_root)?;
        let env_layer = loaders::load_env_layer();

        let mut merged = merge::MergeState::new(default_layer, ConfigSource::Default);
        merged.apply(user_layer, ConfigSource::User);
        merged.apply(project_layer, ConfigSource::Project);
        merged.apply(env_layer, ConfigSource::Environment);
        if let Some(cli) = self.cli_overrides.clone() {
            merged.apply(cli, ConfigSource::Cli);
        }

        let (resolved, layer_provenance, mut merge_warnings) = merged.finalize(&default_paths)?;
        provenance.extend(layer_provenance);

        let mut warnings = validate::validate_config_with_overrides(
            &resolved,
            self.allow_dev,
            self.allow_network,
        )?;
        warnings.append(&mut merge_warnings);

        Ok(ResolvedConfig {
            config: resolved,
            provenance,
            warnings,
        })
    }

    pub fn load_detailed(&self) -> anyhow::Result<ResolvedConfigDetailed> {
        let project_root = self.resolve_project_root()?;

        let default_paths = DefaultPaths::from_root(&project_root);

        let default_layer = loaders::default_layer(&project_root, &default_paths);
        let (user_layer, user_origin) = loaders::load_user_config_with_origin()?;
        let (project_layer, project_origin) = self.load_project_layer(&project_root)?;

        let env_layers = loaders::load_env_layers_detailed();

        let mut merged = merge::MergeStateDetailed::new(
            default_layer,
            merge::ProvenanceCtx::new(ConfigSource::Default, Some("defaults".into())),
        );
        if let Some(origin) = user_origin {
            merged.apply(
                user_layer,
                merge::ProvenanceCtx::new(ConfigSource::User, Some(origin.display().to_string())),
            );
        } else {
            merged.apply(
                user_layer,
                merge::ProvenanceCtx::new(ConfigSource::User, None),
            );
        }
        merged.apply(
            project_layer,
            merge::ProvenanceCtx::new(
                ConfigSource::Project,
                Some(project_origin.display().to_string()),
            ),
        );
        for (layer, env_key) in env_layers {
            merged.apply(
                layer,
                merge::ProvenanceCtx::new(ConfigSource::Environment, Some(env_key)),
            );
        }
        if let Some(cli) = self.cli_overrides.clone() {
            merged.apply(
                cli,
                merge::ProvenanceCtx::new(ConfigSource::Cli, Some("cli".into())),
            );
        }

        let (resolved, provenance, mut merge_warnings) =
            merged.finalize_detailed(&default_paths)?;
        let mut warnings = validate::validate_config_with_overrides(
            &resolved,
            self.allow_dev,
            self.allow_network,
        )?;
        warnings.append(&mut merge_warnings);

        Ok(ResolvedConfigDetailed {
            config: resolved,
            provenance,
            warnings,
        })
    }

    fn resolve_project_root(&self) -> anyhow::Result<PathBuf> {
        let cwd = std::env::current_dir()?;
        Ok(self
            .project_root
            .clone()
            .or_else(|| discover_project_root(&cwd))
            .unwrap_or(cwd))
    }

    fn load_project_layer(
        &self,
        project_root: &std::path::Path,
    ) -> anyhow::Result<(ConfigLayer, PathBuf)> {
        match self.config_path.as_deref() {
            Some(path) => {
                let abs = crate::paths::absolute_path(path)?;
                Ok((loaders::load_config_file_required(&abs)?, abs))
            }
            None => loaders::load_project_config_with_origin(project_root),
        }
    }
}

impl Default for ConfigResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::{
        BackoffLayer, ConfigLayer, DEFAULT_DEPLOYER_BASE_DOMAIN, DeployerLayer, EnvironmentLayer,
        EventsLayer, ServiceEndpointLayer, ServiceTransportLayer, ServicesLayer,
    };
    use greentic_config_types::PackSourceConfig;
    use greentic_types::ConnectionKind;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use url::Url;

    #[test]
    fn precedence_prefers_cli_over_env() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        let env_layer = ConfigLayer {
            environment: Some(EnvironmentLayer {
                env_id: Some(serde_json::from_str("\"staging\"").unwrap()),
                deployment: None,
                connection: None,
                region: None,
            }),
            ..Default::default()
        };

        let cli_layer = ConfigLayer {
            environment: Some(EnvironmentLayer {
                env_id: Some(serde_json::from_str("\"prod\"").unwrap()),
                deployment: None,
                connection: None,
                region: None,
            }),
            ..Default::default()
        };

        let default_paths = DefaultPaths::from_root(&root);
        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(env_layer, ConfigSource::Environment);
        merged.apply(cli_layer, ConfigSource::Cli);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        let env_id_str = serde_json::to_string(&resolved.environment.env_id).unwrap();
        assert!(env_id_str.contains("prod"));
    }

    #[test]
    fn relative_paths_resolve_to_absolute() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let layer = ConfigLayer {
            paths: Some(crate::loaders::PathsLayer {
                state_dir: Some(PathBuf::from("relative/state")),
                cache_dir: Some(PathBuf::from("relative/cache")),
                logs_dir: Some(PathBuf::from("relative/logs")),
                greentic_root: Some(PathBuf::from(".")),
            }),
            packs: Some(crate::loaders::PacksLayer {
                cache_dir: Some(PathBuf::from("relative/packs/cache")),
                source: Some(crate::loaders::PackSourceLayer::LocalIndex {
                    path: Some(PathBuf::from("relative/packs/index.json")),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(layer, ConfigSource::Project);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        assert!(resolved.paths.state_dir.is_absolute());
        assert!(resolved.paths.cache_dir.is_absolute());
        assert!(resolved.paths.logs_dir.is_absolute());
        let packs = resolved.packs.unwrap();
        assert!(packs.cache_dir.is_absolute());
        if let PackSourceConfig::LocalIndex { path } = packs.source {
            assert!(path.is_absolute());
        }
    }

    #[test]
    fn dev_config_requires_dev_env_without_allow() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let layer = ConfigLayer {
            dev: Some(crate::loaders::DevLayer {
                default_env: Some(serde_json::from_str("\"dev\"").unwrap()),
                default_tenant: Some("acme".into()),
                default_team: None,
            }),
            environment: Some(EnvironmentLayer {
                env_id: Some(serde_json::from_str("\"prod\"").unwrap()),
                deployment: None,
                connection: None,
                region: None,
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(layer, ConfigSource::Cli);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        let validation = validate::validate_config(&resolved, false);
        assert!(validation.is_err());
    }

    #[test]
    fn packs_default_to_paths_based_locations() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);
        let cache_dir = root.join("custom_cache");
        let state_dir = root.join(".state_dir");

        let layer = ConfigLayer {
            paths: Some(crate::loaders::PathsLayer {
                cache_dir: Some(cache_dir.clone()),
                state_dir: Some(state_dir.clone()),
                greentic_root: Some(root.clone()),
                logs_dir: None,
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(layer, ConfigSource::Project);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        let packs = resolved.packs.unwrap();
        assert_eq!(packs.cache_dir, cache_dir.join("packs"));
        if let PackSourceConfig::LocalIndex { path } = packs.source {
            assert_eq!(path, state_dir.join("packs").join("index.json"));
        } else {
            panic!("expected local index default");
        }
    }

    #[test]
    fn offline_env_forbids_remote_packs() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let layer = ConfigLayer {
            environment: Some(EnvironmentLayer {
                env_id: Some(serde_json::from_str("\"dev\"").unwrap()),
                deployment: None,
                connection: Some(ConnectionKind::Offline),
                region: None,
            }),
            packs: Some(crate::loaders::PacksLayer {
                source: Some(crate::loaders::PackSourceLayer::HttpIndex {
                    url: Some("https://example.com/index.json".into()),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(layer, ConfigSource::Project);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        let validation = validate::validate_config(&resolved, false);
        assert!(matches!(
            validation,
            Err(validate::ValidationError::PacksSourceOffline)
        ));
    }

    #[test]
    fn events_endpoint_precedence_and_provenance() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);
        let user_layer = ConfigLayer {
            services: Some(ServicesLayer {
                events: Some(ServiceEndpointLayer {
                    url: Some(Url::parse("https://user.example.com").unwrap()),
                    headers: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let project_layer = ConfigLayer {
            services: Some(ServicesLayer {
                events: Some(ServiceEndpointLayer {
                    url: Some(Url::parse("https://project.example.com").unwrap()),
                    headers: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let env_layer = ConfigLayer {
            services: Some(ServicesLayer {
                events: Some(ServiceEndpointLayer {
                    url: Some(Url::parse("https://env.example.com").unwrap()),
                    headers: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let cli_layer = ConfigLayer {
            services: Some(ServicesLayer {
                events: Some(ServiceEndpointLayer {
                    url: Some(Url::parse("https://cli.example.com").unwrap()),
                    headers: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(user_layer, ConfigSource::User);
        merged.apply(project_layer, ConfigSource::Project);
        merged.apply(env_layer, ConfigSource::Environment);
        merged.apply(cli_layer, ConfigSource::Cli);
        let (resolved, provenance, _) = merged.finalize(&default_paths).unwrap();

        let url = resolved.services.unwrap().events.unwrap().url.to_string();
        assert_eq!(url, "https://cli.example.com/");
        assert_eq!(
            provenance
                .get(&greentic_config_types::ProvenancePath(
                    "services.events.url".into()
                ))
                .cloned(),
            Some(ConfigSource::Cli)
        );
    }

    #[test]
    fn runner_transport_precedence_and_provenance() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let user_layer = ConfigLayer {
            services: Some(ServicesLayer {
                runner: Some(ServiceTransportLayer {
                    kind: Some("http".into()),
                    url: Some(Url::parse("https://user-runner.example.com").unwrap()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let project_layer = ConfigLayer {
            services: Some(ServicesLayer {
                runner: Some(ServiceTransportLayer {
                    kind: Some("http".into()),
                    url: Some(Url::parse("https://project-runner.example.com").unwrap()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let env_layer = ConfigLayer {
            services: Some(ServicesLayer {
                runner: Some(ServiceTransportLayer {
                    kind: Some("http".into()),
                    url: Some(Url::parse("https://env-runner.example.com").unwrap()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let cli_layer = ConfigLayer {
            services: Some(ServicesLayer {
                runner: Some(ServiceTransportLayer {
                    kind: Some("http".into()),
                    url: Some(Url::parse("https://cli-runner.example.com").unwrap()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(user_layer, ConfigSource::User);
        merged.apply(project_layer, ConfigSource::Project);
        merged.apply(env_layer, ConfigSource::Environment);
        merged.apply(cli_layer, ConfigSource::Cli);
        let (resolved, provenance, _) = merged.finalize(&default_paths).unwrap();

        let runner_url = match resolved.services.unwrap().runner.as_ref().unwrap() {
            greentic_config_types::ServiceTransportConfig::Http { url, .. }
            | greentic_config_types::ServiceTransportConfig::Nats { url, .. } => url.to_string(),
            greentic_config_types::ServiceTransportConfig::Noop => {
                panic!("expected http or nats transport")
            }
        };
        assert_eq!(runner_url, "https://cli-runner.example.com/");
        assert_eq!(
            provenance
                .get(&greentic_config_types::ProvenancePath(
                    "services.runner.url".into()
                ))
                .cloned(),
            Some(ConfigSource::Cli)
        );
    }

    #[test]
    fn runtime_admin_endpoints_precedence_and_provenance() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let user_layer = ConfigLayer {
            runtime: Some(crate::loaders::RuntimeLayer {
                admin_endpoints: Some(crate::loaders::AdminEndpointsLayer {
                    secrets_explain_enabled: Some(false),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let project_layer = ConfigLayer {
            runtime: Some(crate::loaders::RuntimeLayer {
                admin_endpoints: Some(crate::loaders::AdminEndpointsLayer {
                    secrets_explain_enabled: Some(true),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let env_layer = ConfigLayer {
            runtime: Some(crate::loaders::RuntimeLayer {
                admin_endpoints: Some(crate::loaders::AdminEndpointsLayer {
                    secrets_explain_enabled: Some(false),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let cli_layer = ConfigLayer {
            runtime: Some(crate::loaders::RuntimeLayer {
                admin_endpoints: Some(crate::loaders::AdminEndpointsLayer {
                    secrets_explain_enabled: Some(true),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(user_layer, ConfigSource::User);
        merged.apply(project_layer, ConfigSource::Project);
        merged.apply(env_layer, ConfigSource::Environment);
        merged.apply(cli_layer, ConfigSource::Cli);
        let (resolved, provenance, _) = merged.finalize(&default_paths).unwrap();

        let enabled = resolved
            .runtime
            .admin_endpoints
            .as_ref()
            .map(|a| a.secrets_explain_enabled)
            .unwrap();
        assert!(enabled);
        assert_eq!(
            provenance
                .get(&greentic_config_types::ProvenancePath(
                    "runtime.admin_endpoints.secrets_explain_enabled".into()
                ))
                .cloned(),
            Some(ConfigSource::Cli)
        );
    }

    #[test]
    fn offline_env_blocks_remote_events_endpoint() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let layer = ConfigLayer {
            environment: Some(EnvironmentLayer {
                env_id: Some(serde_json::from_str("\"dev\"").unwrap()),
                deployment: None,
                connection: Some(ConnectionKind::Offline),
                region: None,
            }),
            services: Some(ServicesLayer {
                events: Some(ServiceEndpointLayer {
                    url: Some(Url::parse("https://events.example.com").unwrap()),
                    headers: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(layer, ConfigSource::Project);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        let validation = validate::validate_config(&resolved, false);
        assert!(matches!(
            validation,
            Err(validate::ValidationError::EventsEndpointOffline(_))
        ));
    }

    #[test]
    fn backoff_validation_catches_invalid_values() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let invalid_backoff = ConfigLayer {
            events: Some(EventsLayer {
                backoff: Some(BackoffLayer {
                    initial_ms: Some(0),
                    max_ms: Some(10),
                    multiplier: Some(0.5),
                    jitter: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(invalid_backoff, ConfigSource::Project);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();

        let validation = validate::validate_config(&resolved, true);
        assert!(matches!(
            validation,
            Err(validate::ValidationError::EventsBackoffInitial(0))
        ));

        let invalid_max = ConfigLayer {
            events: Some(EventsLayer {
                backoff: Some(BackoffLayer {
                    initial_ms: Some(200),
                    max_ms: Some(100),
                    multiplier: Some(2.0),
                    jitter: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(invalid_max, ConfigSource::Project);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        let validation = validate::validate_config(&resolved, true);
        assert!(matches!(
            validation,
            Err(validate::ValidationError::EventsBackoffMax { .. })
        ));
    }

    #[test]
    fn deployer_base_domain_defaults_and_validation() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        assert_eq!(
            resolved
                .deployer
                .as_ref()
                .and_then(|d| d.base_domain.as_deref()),
            Some(DEFAULT_DEPLOYER_BASE_DOMAIN)
        );

        let invalid_layer = ConfigLayer {
            deployer: Some(DeployerLayer {
                base_domain: Some("https://bad-domain".into()),
                provider: None,
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(invalid_layer, ConfigSource::Project);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        let validation = validate::validate_config(&resolved, true);
        assert!(matches!(
            validation,
            Err(validate::ValidationError::DeployerBaseDomain(_))
        ));
    }

    #[test]
    fn deployer_base_domain_precedence_and_provenance() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let user_layer = ConfigLayer {
            deployer: Some(DeployerLayer {
                base_domain: Some("user.greentic.test".into()),
                provider: None,
            }),
            ..Default::default()
        };
        let project_layer = ConfigLayer {
            deployer: Some(DeployerLayer {
                base_domain: Some("project.greentic.test".into()),
                provider: None,
            }),
            ..Default::default()
        };
        let env_layer = ConfigLayer {
            deployer: Some(DeployerLayer {
                base_domain: Some("env.greentic.test".into()),
                provider: None,
            }),
            ..Default::default()
        };
        let cli_layer = ConfigLayer {
            deployer: Some(DeployerLayer {
                base_domain: Some("cli.greentic.test".into()),
                provider: None,
            }),
            ..Default::default()
        };

        let mut merged = merge::MergeState::new(
            loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        merged.apply(user_layer, ConfigSource::User);
        merged.apply(project_layer, ConfigSource::Project);
        merged.apply(env_layer, ConfigSource::Environment);
        merged.apply(cli_layer, ConfigSource::Cli);

        let (resolved, provenance, _) = merged.finalize(&default_paths).unwrap();
        let base_domain = resolved
            .deployer
            .as_ref()
            .and_then(|d| d.base_domain.as_deref())
            .unwrap();
        assert_eq!(base_domain, "cli.greentic.test");
        assert_eq!(
            provenance
                .get(&greentic_config_types::ProvenancePath(
                    "deployer.base_domain".into()
                ))
                .cloned(),
            Some(ConfigSource::Cli)
        );
    }

    #[test]
    fn explicit_config_path_replaces_project_discovery() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".greentic")).unwrap();

        std::fs::write(
            root.join(".greentic").join("config.toml"),
            r#"
[environment]
env_id = "staging"
"#,
        )
        .unwrap();

        let explicit_path = root.join("explicit.toml");
        std::fs::write(
            &explicit_path,
            r#"
[environment]
env_id = "prod"
"#,
        )
        .unwrap();

        let resolver = ConfigResolver::new()
            .with_project_root(root.clone())
            .with_config_path(explicit_path.clone());
        let (layer, origin) = resolver.load_project_layer(&root).unwrap();
        assert_eq!(origin, crate::paths::absolute_path(&explicit_path).unwrap());

        let env_id = layer.environment.unwrap().env_id.unwrap();
        let env_json = serde_json::to_string(&env_id).unwrap();
        assert!(env_json.contains("prod"));
    }

    #[test]
    fn explicit_config_path_missing_is_error() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let missing = root.join("missing.toml");

        let resolver = ConfigResolver::new()
            .with_project_root(root.clone())
            .with_config_path(missing.clone());
        let err = resolver.load_project_layer(&root).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("explicit config file not found"));
        assert!(
            msg.contains(
                &crate::paths::absolute_path(&missing)
                    .unwrap()
                    .display()
                    .to_string()
            )
        );
    }

    #[test]
    fn detailed_precedence_prefers_cli_and_tracks_origin() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);

        let base = crate::loaders::default_layer(&root, &default_paths);
        let mut merged = crate::merge::MergeStateDetailed::new(
            base,
            crate::merge::ProvenanceCtx::new(ConfigSource::Default, Some("defaults".into())),
        );

        let user_layer = ConfigLayer {
            paths: Some(crate::loaders::PathsLayer {
                state_dir: Some(PathBuf::from("/tmp/user-state")),
                ..Default::default()
            }),
            ..Default::default()
        };
        merged.apply(
            user_layer,
            crate::merge::ProvenanceCtx::new(ConfigSource::User, Some("/user/config.toml".into())),
        );

        let project_layer = ConfigLayer {
            paths: Some(crate::loaders::PathsLayer {
                state_dir: Some(PathBuf::from("/tmp/project-state")),
                ..Default::default()
            }),
            ..Default::default()
        };
        merged.apply(
            project_layer,
            crate::merge::ProvenanceCtx::new(
                ConfigSource::Project,
                Some("/repo/.greentic/config.toml".into()),
            ),
        );

        let env_layers = crate::loaders::load_env_layers_detailed_from([(
            "GREENTIC_PATHS_STATE_DIR".to_string(),
            "/tmp/env-state".to_string(),
        )]);
        for (layer, key) in env_layers {
            merged.apply(
                layer,
                crate::merge::ProvenanceCtx::new(ConfigSource::Environment, Some(key)),
            );
        }

        let cli_layer = ConfigLayer {
            paths: Some(crate::loaders::PathsLayer {
                state_dir: Some(PathBuf::from("/tmp/cli-state")),
                ..Default::default()
            }),
            ..Default::default()
        };
        merged.apply(
            cli_layer,
            crate::merge::ProvenanceCtx::new(ConfigSource::Cli, Some("cli".into())),
        );

        let (resolved, provenance, _) = merged.finalize_detailed(&default_paths).unwrap();
        assert_eq!(resolved.paths.state_dir, PathBuf::from("/tmp/cli-state"));
        let rec = provenance
            .get(&greentic_config_types::ProvenancePath(
                "paths.state_dir".into(),
            ))
            .unwrap();
        assert_eq!(rec.source, ConfigSource::Cli);
        assert_eq!(rec.origin.as_deref(), Some("cli"));
    }

    #[test]
    fn offline_telemetry_endpoint_warns_unless_allow_network() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);
        let merged = crate::merge::MergeState::new(
            crate::loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        let (mut config, _, _) = merged.finalize(&default_paths).unwrap();
        config.environment.connection = Some(ConnectionKind::Offline);
        config.telemetry.enabled = true;
        config.telemetry.endpoint = Some("https://otlp.example.com:4317".into());
        config.telemetry.exporter = greentic_config_types::TelemetryExporterKind::Otlp;

        let warnings =
            crate::validate::validate_config_with_overrides(&config, true, false).unwrap();
        assert!(warnings.iter().any(|w| w.contains("telemetry.endpoint")));

        let warnings =
            crate::validate::validate_config_with_overrides(&config, true, true).unwrap();
        assert!(!warnings.iter().any(|w| w.contains("telemetry.endpoint")));
    }

    #[test]
    fn offline_events_endpoint_error_is_suppressed_with_allow_network() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);
        let merged = crate::merge::MergeState::new(
            crate::loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );
        let (mut config, _, _) = merged.finalize(&default_paths).unwrap();
        config.environment.connection = Some(ConnectionKind::Offline);
        config.services = Some(greentic_config_types::ServicesConfig {
            events: Some(greentic_config_types::ServiceEndpointConfig {
                url: Url::parse("https://events.example.com").unwrap(),
                headers: None,
            }),
            ..Default::default()
        });

        let err =
            crate::validate::validate_config_with_overrides(&config, true, false).unwrap_err();
        assert!(matches!(
            err,
            crate::validate::ValidationError::EventsEndpointOffline(_)
        ));

        let warnings =
            crate::validate::validate_config_with_overrides(&config, true, true).unwrap();
        assert!(
            !warnings
                .iter()
                .any(|w| w.contains("events endpoint") || w.contains("EventsEndpointOffline"))
        );
    }

    #[test]
    fn offline_service_transport_emits_warning() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let default_paths = DefaultPaths::from_root(&root);
        let mut merged = crate::merge::MergeState::new(
            crate::loaders::default_layer(&root, &default_paths),
            ConfigSource::Default,
        );

        let layer = ConfigLayer {
            environment: Some(EnvironmentLayer {
                env_id: Some(serde_json::from_str("\"dev\"").unwrap()),
                deployment: None,
                connection: Some(ConnectionKind::Offline),
                region: None,
            }),
            services: Some(ServicesLayer {
                runner: Some(ServiceTransportLayer {
                    kind: Some("http".into()),
                    url: Some(Url::parse("https://runner.example.com").unwrap()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        merged.apply(layer, ConfigSource::Project);
        let (resolved, _, _) = merged.finalize(&default_paths).unwrap();
        let warnings = crate::validate::validate_config_with_overrides(&resolved, true, false)
            .expect("validation should warn, not error");
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("services.runner") && w.contains("offline"))
        );
    }
}
