mod explain;
mod loaders;
mod merge;
mod paths;
mod validate;

use greentic_config_types::{ConfigSource, GreenticConfig, ProvenancePath};
use std::collections::HashMap;
use std::path::PathBuf;

pub use explain::{ExplainReport, explain};
pub use loaders::{ConfigFileFormat, ConfigLayer};
pub use paths::{DefaultPaths, discover_project_root};
pub use validate::{ValidationError, validate_config};

pub type ProvenanceMap = HashMap<ProvenancePath, ConfigSource>;

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub config: GreenticConfig,
    pub provenance: ProvenanceMap,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigResolver {
    project_root: Option<PathBuf>,
    cli_overrides: Option<ConfigLayer>,
    allow_dev: bool,
}

impl ConfigResolver {
    pub fn new() -> Self {
        Self {
            project_root: None,
            cli_overrides: None,
            allow_dev: false,
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

    pub fn with_cli_overrides(mut self, layer: ConfigLayer) -> Self {
        self.cli_overrides = Some(layer);
        self
    }

    pub fn allow_dev(mut self, allow: bool) -> Self {
        self.allow_dev = allow;
        self
    }

    pub fn load(&self) -> anyhow::Result<ResolvedConfig> {
        let cwd = std::env::current_dir()?;
        let project_root = self
            .project_root
            .clone()
            .or_else(|| discover_project_root(&cwd))
            .unwrap_or(cwd);

        let default_paths = DefaultPaths::from_root(&project_root);
        let mut provenance = ProvenanceMap::new();

        let default_layer = loaders::default_layer(&project_root, &default_paths);
        let user_layer = loaders::load_user_config()?;
        let project_layer = loaders::load_project_config(&project_root)?;
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

        let mut warnings = validate::validate_config(&resolved, self.allow_dev)?;
        warnings.append(&mut merge_warnings);

        Ok(ResolvedConfig {
            config: resolved,
            provenance,
            warnings,
        })
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
        EventsLayer, ServiceEndpointLayer, ServicesLayer,
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
            }),
            ..Default::default()
        };
        let project_layer = ConfigLayer {
            services: Some(ServicesLayer {
                events: Some(ServiceEndpointLayer {
                    url: Some(Url::parse("https://project.example.com").unwrap()),
                    headers: None,
                }),
            }),
            ..Default::default()
        };
        let env_layer = ConfigLayer {
            services: Some(ServicesLayer {
                events: Some(ServiceEndpointLayer {
                    url: Some(Url::parse("https://env.example.com").unwrap()),
                    headers: None,
                }),
            }),
            ..Default::default()
        };
        let cli_layer = ConfigLayer {
            services: Some(ServicesLayer {
                events: Some(ServiceEndpointLayer {
                    url: Some(Url::parse("https://cli.example.com").unwrap()),
                    headers: None,
                }),
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
                    "services.events".into()
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
}
