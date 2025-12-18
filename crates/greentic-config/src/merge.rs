use crate::loaders::ConfigLayer;
use crate::loaders::{
    DEFAULT_DEPLOYER_BASE_DOMAIN, EnvironmentLayer, PackSourceLayer, PacksLayer, default_env_id,
};
use crate::paths::DefaultPaths;
use crate::{ProvenanceMap, ProvenanceMapDetailed, ProvenanceRecord};
use greentic_config_types::{
    BackoffConfig, ConfigSource, ConfigVersion, DeployerConfig, DeployerProviderDefaults,
    DevConfig, EnvironmentConfig, EventsConfig, GreenticConfig, NetworkConfig, PackSourceConfig,
    PackTrustConfig, PacksConfig, PathsConfig, ProvenancePath, ReconnectConfig, RuntimeConfig,
    SecretsBackendRefConfig, ServiceEndpointConfig, ServicesConfig, TelemetryConfig,
    TelemetryExporterKind, TlsMode,
};
use std::path::PathBuf;

pub struct MergeState {
    acc: ConfigLayer,
    provenance: ProvenanceMap,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProvenanceCtx {
    pub source: ConfigSource,
    pub origin: Option<String>,
}

impl ProvenanceCtx {
    pub fn new(source: ConfigSource, origin: Option<String>) -> Self {
        Self { source, origin }
    }
}

impl MergeState {
    pub fn new(base: ConfigLayer, source: ConfigSource) -> Self {
        let mut state = Self {
            acc: ConfigLayer::default(),
            provenance: ProvenanceMap::new(),
            warnings: Vec::new(),
        };
        state.apply(base, source);
        state
    }

    pub fn apply(&mut self, layer: ConfigLayer, source: ConfigSource) {
        if let Some(schema_version) = layer.schema_version {
            self.acc.schema_version = Some(schema_version.clone());
            self.provenance
                .insert(ProvenancePath("schema_version".into()), source.clone());
        }

        if let Some(env) = layer.environment {
            let target = self.acc.environment.get_or_insert_with(Default::default);
            set_field(
                &mut target.env_id,
                env.env_id,
                &mut self.provenance,
                "environment.env_id",
                &source,
            );
            set_field(
                &mut target.deployment,
                env.deployment,
                &mut self.provenance,
                "environment.deployment",
                &source,
            );
            set_field(
                &mut target.connection,
                env.connection,
                &mut self.provenance,
                "environment.connection",
                &source,
            );
            set_field(
                &mut target.region,
                env.region,
                &mut self.provenance,
                "environment.region",
                &source,
            );
        }

        if let Some(paths) = layer.paths {
            let target = self.acc.paths.get_or_insert_with(Default::default);
            set_field(
                &mut target.greentic_root,
                paths.greentic_root,
                &mut self.provenance,
                "paths.greentic_root",
                &source,
            );
            set_field(
                &mut target.state_dir,
                paths.state_dir,
                &mut self.provenance,
                "paths.state_dir",
                &source,
            );
            set_field(
                &mut target.cache_dir,
                paths.cache_dir,
                &mut self.provenance,
                "paths.cache_dir",
                &source,
            );
            set_field(
                &mut target.logs_dir,
                paths.logs_dir,
                &mut self.provenance,
                "paths.logs_dir",
                &source,
            );
        }

        if let Some(services) = layer.services {
            let target = self.acc.services.get_or_insert_with(Default::default);
            if let Some(events) = services.events {
                target.events = Some(events.clone());
                self.provenance
                    .insert(ProvenancePath("services.events".into()), source.clone());
            }
        }

        if let Some(events) = layer.events {
            let target = self.acc.events.get_or_insert_with(Default::default);
            set_field(
                &mut target.reconnect,
                events.reconnect,
                &mut self.provenance,
                "events.reconnect",
                &source,
            );
            set_field(
                &mut target.backoff,
                events.backoff,
                &mut self.provenance,
                "events.backoff",
                &source,
            );
        }

        if let Some(runtime) = layer.runtime {
            let target = self.acc.runtime.get_or_insert_with(Default::default);
            set_field(
                &mut target.max_concurrency,
                runtime.max_concurrency,
                &mut self.provenance,
                "runtime.max_concurrency",
                &source,
            );
            set_field(
                &mut target.task_timeout_ms,
                runtime.task_timeout_ms,
                &mut self.provenance,
                "runtime.task_timeout_ms",
                &source,
            );
            set_field(
                &mut target.shutdown_grace_ms,
                runtime.shutdown_grace_ms,
                &mut self.provenance,
                "runtime.shutdown_grace_ms",
                &source,
            );
        }

        if let Some(telemetry) = layer.telemetry {
            let target = self.acc.telemetry.get_or_insert_with(Default::default);
            set_field(
                &mut target.enabled,
                telemetry.enabled,
                &mut self.provenance,
                "telemetry.enabled",
                &source,
            );
            set_field(
                &mut target.exporter,
                telemetry.exporter,
                &mut self.provenance,
                "telemetry.exporter",
                &source,
            );
            set_field(
                &mut target.endpoint,
                telemetry.endpoint,
                &mut self.provenance,
                "telemetry.endpoint",
                &source,
            );
            set_field(
                &mut target.sampling,
                telemetry.sampling,
                &mut self.provenance,
                "telemetry.sampling",
                &source,
            );
        }

        if let Some(network) = layer.network {
            let target = self.acc.network.get_or_insert_with(Default::default);
            set_field(
                &mut target.proxy_url,
                network.proxy_url,
                &mut self.provenance,
                "network.proxy_url",
                &source,
            );
            set_field(
                &mut target.tls_mode,
                network.tls_mode,
                &mut self.provenance,
                "network.tls_mode",
                &source,
            );
            set_field(
                &mut target.connect_timeout_ms,
                network.connect_timeout_ms,
                &mut self.provenance,
                "network.connect_timeout_ms",
                &source,
            );
            set_field(
                &mut target.read_timeout_ms,
                network.read_timeout_ms,
                &mut self.provenance,
                "network.read_timeout_ms",
                &source,
            );
        }

        if let Some(deployer) = layer.deployer {
            let target = self.acc.deployer.get_or_insert_with(Default::default);
            set_field(
                &mut target.base_domain,
                deployer.base_domain,
                &mut self.provenance,
                "deployer.base_domain",
                &source,
            );
            set_field(
                &mut target.provider,
                deployer.provider,
                &mut self.provenance,
                "deployer.provider",
                &source,
            );
        }

        if let Some(secrets) = layer.secrets {
            let target = self.acc.secrets.get_or_insert_with(Default::default);
            set_field(
                &mut target.kind,
                secrets.kind,
                &mut self.provenance,
                "secrets.kind",
                &source,
            );
            set_field(
                &mut target.reference,
                secrets.reference,
                &mut self.provenance,
                "secrets.reference",
                &source,
            );
        }

        if let Some(packs) = layer.packs {
            let target = self.acc.packs.get_or_insert_with(Default::default);
            set_field(
                &mut target.cache_dir,
                packs.cache_dir,
                &mut self.provenance,
                "packs.cache_dir",
                &source,
            );
            set_field(
                &mut target.index_cache_ttl_secs,
                packs.index_cache_ttl_secs,
                &mut self.provenance,
                "packs.index_cache_ttl_secs",
                &source,
            );
            if let Some(source_layer) = packs.source {
                target.source = Some(source_layer.clone());
                self.provenance
                    .insert(ProvenancePath("packs.source".into()), source.clone());
            }
            if let Some(trust) = packs.trust {
                let trust_target = target.trust.get_or_insert_with(Default::default);
                set_field(
                    &mut trust_target.public_keys,
                    trust.public_keys,
                    &mut self.provenance,
                    "packs.trust.public_keys",
                    &source,
                );
                set_field(
                    &mut trust_target.require_signatures,
                    trust.require_signatures,
                    &mut self.provenance,
                    "packs.trust.require_signatures",
                    &source,
                );
            }
        }

        if let Some(dev) = layer.dev {
            let target = self.acc.dev.get_or_insert_with(Default::default);
            set_field(
                &mut target.default_env,
                dev.default_env,
                &mut self.provenance,
                "dev.default_env",
                &source,
            );
            set_field(
                &mut target.default_tenant,
                dev.default_tenant,
                &mut self.provenance,
                "dev.default_tenant",
                &source,
            );
            set_field(
                &mut target.default_team,
                dev.default_team,
                &mut self.provenance,
                "dev.default_team",
                &source,
            );
        }
    }

    pub fn finalize(
        mut self,
        defaults: &DefaultPaths,
    ) -> anyhow::Result<(GreenticConfig, ProvenanceMap, Vec<String>)> {
        let schema_version = self
            .acc
            .schema_version
            .take()
            .unwrap_or_else(ConfigVersion::v1);

        let env_layer: EnvironmentLayer = self.acc.environment.take().unwrap_or_default();
        let env_id = env_layer.env_id.unwrap_or_else(default_env_id);
        let environment = EnvironmentConfig {
            env_id,
            deployment: env_layer.deployment,
            connection: env_layer.connection,
            region: env_layer.region,
        };

        let paths_layer = self.acc.paths.take().unwrap_or_default();
        let greentic_root = make_absolute(
            paths_layer
                .greentic_root
                .unwrap_or_else(|| defaults.root.clone()),
            defaults,
        );
        let state_dir = make_absolute(
            paths_layer
                .state_dir
                .unwrap_or_else(|| defaults.state_dir.clone()),
            defaults,
        );
        let cache_dir = make_absolute(
            paths_layer
                .cache_dir
                .unwrap_or_else(|| defaults.cache_dir.clone()),
            defaults,
        );
        let logs_dir = make_absolute(
            paths_layer
                .logs_dir
                .unwrap_or_else(|| defaults.logs_dir.clone()),
            defaults,
        );

        let paths = PathsConfig {
            greentic_root,
            state_dir,
            cache_dir,
            logs_dir,
        };

        let services_layer = self.acc.services.take().unwrap_or_default();
        let services = services_layer
            .events
            .map(|evt| -> anyhow::Result<ServicesConfig> {
                let url = evt.url.ok_or_else(|| {
                    anyhow::anyhow!("services.events.url is required when events are configured")
                })?;
                Ok(ServicesConfig {
                    events: Some(ServiceEndpointConfig {
                        url,
                        headers: evt.headers,
                    }),
                    ..Default::default()
                })
            })
            .transpose()?;

        let events_layer = self.acc.events.take().unwrap_or_default();
        let reconnect_layer = events_layer.reconnect.unwrap_or_default();
        let backoff_layer = events_layer.backoff.unwrap_or_default();
        let events = EventsConfig {
            reconnect: Some(ReconnectConfig {
                enabled: Some(reconnect_layer.enabled.unwrap_or(true)),
                max_retries: reconnect_layer.max_retries.or(Some(50)),
            }),
            backoff: Some(BackoffConfig {
                initial_ms: Some(backoff_layer.initial_ms.unwrap_or(250)),
                max_ms: Some(backoff_layer.max_ms.unwrap_or(30_000)),
                multiplier: Some(backoff_layer.multiplier.unwrap_or(2.0)),
                jitter: Some(backoff_layer.jitter.unwrap_or(true)),
            }),
        };

        let runtime_layer = self.acc.runtime.take().unwrap_or_default();
        let runtime = RuntimeConfig {
            max_concurrency: runtime_layer.max_concurrency,
            task_timeout_ms: runtime_layer.task_timeout_ms,
            shutdown_grace_ms: runtime_layer.shutdown_grace_ms,
            ..Default::default()
        };

        let telemetry_layer = self.acc.telemetry.take().unwrap_or_default();
        let (exporter, exporter_warning) = parse_exporter(telemetry_layer.exporter);
        if let Some(warn) = exporter_warning {
            self.warnings.push(warn);
        }

        let telemetry = TelemetryConfig {
            enabled: telemetry_layer.enabled.unwrap_or(true),
            exporter,
            endpoint: telemetry_layer.endpoint,
            sampling: telemetry_layer.sampling.unwrap_or(1.0),
        };

        let network_layer = self.acc.network.take().unwrap_or_default();
        let (tls_mode, tls_warning) = parse_tls_mode(network_layer.tls_mode);
        if let Some(warn) = tls_warning {
            self.warnings.push(warn);
        }
        let network = NetworkConfig {
            proxy_url: network_layer.proxy_url,
            tls_mode,
            connect_timeout_ms: network_layer.connect_timeout_ms,
            read_timeout_ms: network_layer.read_timeout_ms,
        };

        let deployer_layer = self.acc.deployer.take().unwrap_or_default();
        let deployer = Some(DeployerConfig {
            base_domain: Some(
                deployer_layer
                    .base_domain
                    .unwrap_or_else(|| DEFAULT_DEPLOYER_BASE_DOMAIN.to_string()),
            ),
            provider: deployer_layer
                .provider
                .map(|provider| DeployerProviderDefaults {
                    provider_kind: provider.provider_kind,
                    region: provider.region,
                }),
        });

        let secrets_layer = self.acc.secrets.take().unwrap_or_default();
        let secrets = SecretsBackendRefConfig {
            kind: secrets_layer.kind.unwrap_or_else(|| "none".into()),
            reference: secrets_layer.reference,
        };

        let packs_layer = self.acc.packs.take().unwrap_or_default();
        let default_pack_index = default_pack_index_path(&paths, defaults);
        let (packs, mut pack_warnings) =
            finalize_packs(packs_layer, &paths, &default_pack_index, defaults)?;
        self.warnings.append(&mut pack_warnings);

        let dev = self.acc.dev.take().and_then(|dev_layer| {
            let env = dev_layer
                .default_env
                .or_else(|| Some(default_env_id()))
                .unwrap();
            let tenant = dev_layer.default_tenant?;
            Some(DevConfig {
                default_env: env,
                default_tenant: tenant,
                default_team: dev_layer.default_team,
            })
        });

        let config = GreenticConfig {
            schema_version,
            environment,
            paths,
            packs: Some(packs),
            services,
            events: Some(events),
            runtime,
            telemetry,
            network,
            deployer,
            secrets,
            dev,
        };

        Ok((config, self.provenance, self.warnings))
    }
}

fn set_field<T: Clone>(
    target: &mut Option<T>,
    incoming: Option<T>,
    provenance: &mut ProvenanceMap,
    path: &str,
    source: &ConfigSource,
) {
    if let Some(value) = incoming {
        *target = Some(value);
        provenance.insert(ProvenancePath(path.to_string()), source.clone());
    }
}

fn set_field_detailed<T: Clone>(
    target: &mut Option<T>,
    incoming: Option<T>,
    provenance: &mut ProvenanceMapDetailed,
    path: &str,
    ctx: &ProvenanceCtx,
) {
    if let Some(value) = incoming {
        *target = Some(value);
        provenance.insert(
            ProvenancePath(path.to_string()),
            ProvenanceRecord {
                source: ctx.source.clone(),
                origin: ctx.origin.clone(),
            },
        );
    }
}

pub struct MergeStateDetailed {
    acc: ConfigLayer,
    provenance: ProvenanceMapDetailed,
    pub warnings: Vec<String>,
}

impl MergeStateDetailed {
    pub fn new(base: ConfigLayer, ctx: ProvenanceCtx) -> Self {
        let mut state = Self {
            acc: ConfigLayer::default(),
            provenance: ProvenanceMapDetailed::new(),
            warnings: Vec::new(),
        };
        state.apply(base, ctx);
        state
    }

    pub fn apply(&mut self, layer: ConfigLayer, ctx: ProvenanceCtx) {
        if let Some(schema_version) = layer.schema_version {
            self.acc.schema_version = Some(schema_version.clone());
            self.provenance.insert(
                ProvenancePath("schema_version".into()),
                ProvenanceRecord {
                    source: ctx.source.clone(),
                    origin: ctx.origin.clone(),
                },
            );
        }

        if let Some(env) = layer.environment {
            let target = self.acc.environment.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.env_id,
                env.env_id,
                &mut self.provenance,
                "environment.env_id",
                &ctx,
            );
            set_field_detailed(
                &mut target.deployment,
                env.deployment,
                &mut self.provenance,
                "environment.deployment",
                &ctx,
            );
            set_field_detailed(
                &mut target.connection,
                env.connection,
                &mut self.provenance,
                "environment.connection",
                &ctx,
            );
            set_field_detailed(
                &mut target.region,
                env.region,
                &mut self.provenance,
                "environment.region",
                &ctx,
            );
        }

        if let Some(paths) = layer.paths {
            let target = self.acc.paths.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.greentic_root,
                paths.greentic_root,
                &mut self.provenance,
                "paths.greentic_root",
                &ctx,
            );
            set_field_detailed(
                &mut target.state_dir,
                paths.state_dir,
                &mut self.provenance,
                "paths.state_dir",
                &ctx,
            );
            set_field_detailed(
                &mut target.cache_dir,
                paths.cache_dir,
                &mut self.provenance,
                "paths.cache_dir",
                &ctx,
            );
            set_field_detailed(
                &mut target.logs_dir,
                paths.logs_dir,
                &mut self.provenance,
                "paths.logs_dir",
                &ctx,
            );
        }

        if let Some(services) = layer.services {
            let target = self.acc.services.get_or_insert_with(Default::default);
            if let Some(events) = services.events {
                let endpoint = target.events.get_or_insert_with(Default::default);
                set_field_detailed(
                    &mut endpoint.url,
                    events.url,
                    &mut self.provenance,
                    "services.events.url",
                    &ctx,
                );
                set_field_detailed(
                    &mut endpoint.headers,
                    events.headers,
                    &mut self.provenance,
                    "services.events.headers",
                    &ctx,
                );
            }
        }

        if let Some(events) = layer.events {
            let target = self.acc.events.get_or_insert_with(Default::default);
            if let Some(reconnect) = events.reconnect {
                let reconnect_target = target.reconnect.get_or_insert_with(Default::default);
                set_field_detailed(
                    &mut reconnect_target.enabled,
                    reconnect.enabled,
                    &mut self.provenance,
                    "events.reconnect.enabled",
                    &ctx,
                );
                set_field_detailed(
                    &mut reconnect_target.max_retries,
                    reconnect.max_retries,
                    &mut self.provenance,
                    "events.reconnect.max_retries",
                    &ctx,
                );
            }
            if let Some(backoff) = events.backoff {
                let backoff_target = target.backoff.get_or_insert_with(Default::default);
                set_field_detailed(
                    &mut backoff_target.initial_ms,
                    backoff.initial_ms,
                    &mut self.provenance,
                    "events.backoff.initial_ms",
                    &ctx,
                );
                set_field_detailed(
                    &mut backoff_target.max_ms,
                    backoff.max_ms,
                    &mut self.provenance,
                    "events.backoff.max_ms",
                    &ctx,
                );
                set_field_detailed(
                    &mut backoff_target.multiplier,
                    backoff.multiplier,
                    &mut self.provenance,
                    "events.backoff.multiplier",
                    &ctx,
                );
                set_field_detailed(
                    &mut backoff_target.jitter,
                    backoff.jitter,
                    &mut self.provenance,
                    "events.backoff.jitter",
                    &ctx,
                );
            }
        }

        if let Some(runtime) = layer.runtime {
            let target = self.acc.runtime.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.max_concurrency,
                runtime.max_concurrency,
                &mut self.provenance,
                "runtime.max_concurrency",
                &ctx,
            );
            set_field_detailed(
                &mut target.task_timeout_ms,
                runtime.task_timeout_ms,
                &mut self.provenance,
                "runtime.task_timeout_ms",
                &ctx,
            );
            set_field_detailed(
                &mut target.shutdown_grace_ms,
                runtime.shutdown_grace_ms,
                &mut self.provenance,
                "runtime.shutdown_grace_ms",
                &ctx,
            );
        }

        if let Some(telemetry) = layer.telemetry {
            let target = self.acc.telemetry.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.enabled,
                telemetry.enabled,
                &mut self.provenance,
                "telemetry.enabled",
                &ctx,
            );
            set_field_detailed(
                &mut target.exporter,
                telemetry.exporter,
                &mut self.provenance,
                "telemetry.exporter",
                &ctx,
            );
            set_field_detailed(
                &mut target.endpoint,
                telemetry.endpoint,
                &mut self.provenance,
                "telemetry.endpoint",
                &ctx,
            );
            set_field_detailed(
                &mut target.sampling,
                telemetry.sampling,
                &mut self.provenance,
                "telemetry.sampling",
                &ctx,
            );
        }

        if let Some(network) = layer.network {
            let target = self.acc.network.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.proxy_url,
                network.proxy_url,
                &mut self.provenance,
                "network.proxy_url",
                &ctx,
            );
            set_field_detailed(
                &mut target.tls_mode,
                network.tls_mode,
                &mut self.provenance,
                "network.tls_mode",
                &ctx,
            );
            set_field_detailed(
                &mut target.connect_timeout_ms,
                network.connect_timeout_ms,
                &mut self.provenance,
                "network.connect_timeout_ms",
                &ctx,
            );
            set_field_detailed(
                &mut target.read_timeout_ms,
                network.read_timeout_ms,
                &mut self.provenance,
                "network.read_timeout_ms",
                &ctx,
            );
        }

        if let Some(deployer) = layer.deployer {
            let target = self.acc.deployer.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.base_domain,
                deployer.base_domain,
                &mut self.provenance,
                "deployer.base_domain",
                &ctx,
            );
            if let Some(provider) = deployer.provider {
                let provider_target = target.provider.get_or_insert_with(Default::default);
                set_field_detailed(
                    &mut provider_target.provider_kind,
                    provider.provider_kind,
                    &mut self.provenance,
                    "deployer.provider.provider_kind",
                    &ctx,
                );
                set_field_detailed(
                    &mut provider_target.region,
                    provider.region,
                    &mut self.provenance,
                    "deployer.provider.region",
                    &ctx,
                );
            }
        }

        if let Some(secrets) = layer.secrets {
            let target = self.acc.secrets.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.kind,
                secrets.kind,
                &mut self.provenance,
                "secrets.kind",
                &ctx,
            );
            set_field_detailed(
                &mut target.reference,
                secrets.reference,
                &mut self.provenance,
                "secrets.reference",
                &ctx,
            );
        }

        if let Some(packs) = layer.packs {
            let target = self.acc.packs.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.cache_dir,
                packs.cache_dir,
                &mut self.provenance,
                "packs.cache_dir",
                &ctx,
            );
            set_field_detailed(
                &mut target.index_cache_ttl_secs,
                packs.index_cache_ttl_secs,
                &mut self.provenance,
                "packs.index_cache_ttl_secs",
                &ctx,
            );
            if let Some(source_layer) = packs.source {
                target.source = Some(source_layer.clone());
                self.provenance.insert(
                    ProvenancePath("packs.source.type".into()),
                    ProvenanceRecord {
                        source: ctx.source.clone(),
                        origin: ctx.origin.clone(),
                    },
                );
                match source_layer {
                    PackSourceLayer::LocalIndex { path } => {
                        if path.is_some() {
                            self.provenance.insert(
                                ProvenancePath("packs.source.path".into()),
                                ProvenanceRecord {
                                    source: ctx.source.clone(),
                                    origin: ctx.origin.clone(),
                                },
                            );
                        }
                    }
                    PackSourceLayer::HttpIndex { url } => {
                        if url.is_some() {
                            self.provenance.insert(
                                ProvenancePath("packs.source.url".into()),
                                ProvenanceRecord {
                                    source: ctx.source.clone(),
                                    origin: ctx.origin.clone(),
                                },
                            );
                        }
                    }
                    PackSourceLayer::OciRegistry { reference } => {
                        if reference.is_some() {
                            self.provenance.insert(
                                ProvenancePath("packs.source.reference".into()),
                                ProvenanceRecord {
                                    source: ctx.source.clone(),
                                    origin: ctx.origin.clone(),
                                },
                            );
                        }
                    }
                }
            }
            if let Some(trust) = packs.trust {
                let trust_target = target.trust.get_or_insert_with(Default::default);
                set_field_detailed(
                    &mut trust_target.public_keys,
                    trust.public_keys,
                    &mut self.provenance,
                    "packs.trust.public_keys",
                    &ctx,
                );
                set_field_detailed(
                    &mut trust_target.require_signatures,
                    trust.require_signatures,
                    &mut self.provenance,
                    "packs.trust.require_signatures",
                    &ctx,
                );
            }
        }

        if let Some(dev) = layer.dev {
            let target = self.acc.dev.get_or_insert_with(Default::default);
            set_field_detailed(
                &mut target.default_env,
                dev.default_env,
                &mut self.provenance,
                "dev.default_env",
                &ctx,
            );
            set_field_detailed(
                &mut target.default_tenant,
                dev.default_tenant,
                &mut self.provenance,
                "dev.default_tenant",
                &ctx,
            );
            set_field_detailed(
                &mut target.default_team,
                dev.default_team,
                &mut self.provenance,
                "dev.default_team",
                &ctx,
            );
        }
    }

    pub fn finalize_detailed(
        mut self,
        defaults: &DefaultPaths,
    ) -> anyhow::Result<(GreenticConfig, ProvenanceMapDetailed, Vec<String>)> {
        let schema_version = self
            .acc
            .schema_version
            .take()
            .unwrap_or_else(ConfigVersion::v1);

        let env_layer: EnvironmentLayer = self.acc.environment.take().unwrap_or_default();
        let env_id = env_layer.env_id.unwrap_or_else(default_env_id);
        let environment = EnvironmentConfig {
            env_id,
            deployment: env_layer.deployment,
            connection: env_layer.connection,
            region: env_layer.region,
        };

        let paths_layer = self.acc.paths.take().unwrap_or_default();
        let greentic_root = make_absolute(
            paths_layer
                .greentic_root
                .unwrap_or_else(|| defaults.root.clone()),
            defaults,
        );
        let state_dir = make_absolute(
            paths_layer
                .state_dir
                .unwrap_or_else(|| defaults.state_dir.clone()),
            defaults,
        );
        let cache_dir = make_absolute(
            paths_layer
                .cache_dir
                .unwrap_or_else(|| defaults.cache_dir.clone()),
            defaults,
        );
        let logs_dir = make_absolute(
            paths_layer
                .logs_dir
                .unwrap_or_else(|| defaults.logs_dir.clone()),
            defaults,
        );

        let paths = PathsConfig {
            greentic_root,
            state_dir,
            cache_dir,
            logs_dir,
        };

        let services_layer = self.acc.services.take().unwrap_or_default();
        let services = services_layer
            .events
            .map(|evt| -> anyhow::Result<ServicesConfig> {
                let url = evt.url.ok_or_else(|| {
                    anyhow::anyhow!("services.events.url is required when events are configured")
                })?;
                Ok(ServicesConfig {
                    events: Some(ServiceEndpointConfig {
                        url,
                        headers: evt.headers,
                    }),
                    ..Default::default()
                })
            })
            .transpose()?;

        let events_layer = self.acc.events.take().unwrap_or_default();
        let reconnect_layer = events_layer.reconnect.unwrap_or_default();
        let backoff_layer = events_layer.backoff.unwrap_or_default();
        let events = EventsConfig {
            reconnect: Some(ReconnectConfig {
                enabled: Some(reconnect_layer.enabled.unwrap_or(true)),
                max_retries: reconnect_layer.max_retries.or(Some(50)),
            }),
            backoff: Some(BackoffConfig {
                initial_ms: Some(backoff_layer.initial_ms.unwrap_or(250)),
                max_ms: Some(backoff_layer.max_ms.unwrap_or(30_000)),
                multiplier: Some(backoff_layer.multiplier.unwrap_or(2.0)),
                jitter: Some(backoff_layer.jitter.unwrap_or(true)),
            }),
        };

        let runtime_layer = self.acc.runtime.take().unwrap_or_default();
        let runtime = RuntimeConfig {
            max_concurrency: runtime_layer.max_concurrency,
            task_timeout_ms: runtime_layer.task_timeout_ms,
            shutdown_grace_ms: runtime_layer.shutdown_grace_ms,
            ..Default::default()
        };

        let telemetry_layer = self.acc.telemetry.take().unwrap_or_default();
        let (exporter, exporter_warning) = parse_exporter(telemetry_layer.exporter);
        if let Some(warn) = exporter_warning {
            self.warnings.push(warn);
        }

        let telemetry = TelemetryConfig {
            enabled: telemetry_layer.enabled.unwrap_or(true),
            exporter,
            endpoint: telemetry_layer.endpoint,
            sampling: telemetry_layer.sampling.unwrap_or(1.0),
        };

        let network_layer = self.acc.network.take().unwrap_or_default();
        let (tls_mode, tls_warning) = parse_tls_mode(network_layer.tls_mode);
        if let Some(warn) = tls_warning {
            self.warnings.push(warn);
        }
        let network = NetworkConfig {
            proxy_url: network_layer.proxy_url,
            tls_mode,
            connect_timeout_ms: network_layer.connect_timeout_ms,
            read_timeout_ms: network_layer.read_timeout_ms,
        };

        let deployer_layer = self.acc.deployer.take().unwrap_or_default();
        let deployer = Some(DeployerConfig {
            base_domain: Some(
                deployer_layer
                    .base_domain
                    .unwrap_or_else(|| DEFAULT_DEPLOYER_BASE_DOMAIN.to_string()),
            ),
            provider: deployer_layer
                .provider
                .map(|provider| DeployerProviderDefaults {
                    provider_kind: provider.provider_kind,
                    region: provider.region,
                }),
        });

        let secrets_layer = self.acc.secrets.take().unwrap_or_default();
        let secrets = SecretsBackendRefConfig {
            kind: secrets_layer.kind.unwrap_or_else(|| "none".into()),
            reference: secrets_layer.reference,
        };

        let packs_layer = self.acc.packs.take().unwrap_or_default();
        let default_pack_index = default_pack_index_path(&paths, defaults);
        let (packs, mut pack_warnings) =
            finalize_packs(packs_layer, &paths, &default_pack_index, defaults)?;
        self.warnings.append(&mut pack_warnings);

        let dev = self.acc.dev.take().and_then(|dev_layer| {
            let env = dev_layer
                .default_env
                .or_else(|| Some(default_env_id()))
                .unwrap();
            let tenant = dev_layer.default_tenant?;
            Some(DevConfig {
                default_env: env,
                default_tenant: tenant,
                default_team: dev_layer.default_team,
            })
        });

        let config = GreenticConfig {
            schema_version,
            environment,
            paths,
            packs: Some(packs),
            services,
            events: Some(events),
            runtime,
            telemetry,
            network,
            deployer,
            secrets,
            dev,
        };

        ensure_packs_provenance(
            &mut self.provenance,
            config.packs.as_ref().expect("packs are always populated"),
        );

        Ok((config, self.provenance, self.warnings))
    }
}

fn ensure_packs_provenance(provenance: &mut ProvenanceMapDetailed, packs: &PacksConfig) {
    let defaults = ProvenanceRecord {
        source: ConfigSource::Default,
        origin: Some("defaults".into()),
    };
    provenance
        .entry(ProvenancePath("packs.cache_dir".into()))
        .or_insert_with(|| defaults.clone());
    provenance
        .entry(ProvenancePath("packs.source.type".into()))
        .or_insert_with(|| defaults.clone());

    match &packs.source {
        PackSourceConfig::LocalIndex { .. } => {
            provenance
                .entry(ProvenancePath("packs.source.path".into()))
                .or_insert_with(|| defaults.clone());
        }
        PackSourceConfig::HttpIndex { .. } => {
            provenance
                .entry(ProvenancePath("packs.source.url".into()))
                .or_insert_with(|| defaults.clone());
        }
        PackSourceConfig::OciRegistry { .. } => {
            provenance
                .entry(ProvenancePath("packs.source.reference".into()))
                .or_insert_with(|| defaults.clone());
        }
    }
}

fn make_absolute(path: PathBuf, defaults: &DefaultPaths) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        defaults.root.join(path)
    }
}

fn parse_exporter(raw: Option<String>) -> (TelemetryExporterKind, Option<String>) {
    match raw.as_deref() {
        Some("otlp") => (TelemetryExporterKind::Otlp, None),
        Some("stdout") => (TelemetryExporterKind::Stdout, None),
        Some("none") | Some("off") | Some("disabled") => (TelemetryExporterKind::None, None),
        None => (TelemetryExporterKind::None, None),
        Some(other) => (
            TelemetryExporterKind::None,
            Some(format!(
                "Unknown telemetry exporter '{other}', defaulting to none"
            )),
        ),
    }
}

fn parse_tls_mode(raw: Option<String>) -> (TlsMode, Option<String>) {
    match raw.as_deref() {
        Some("disabled") | Some("off") | Some("false") => (TlsMode::Disabled, None),
        Some("strict") => (TlsMode::Strict, None),
        Some("system") | None => (TlsMode::System, None),
        Some(other) => (
            TlsMode::System,
            Some(format!("Unknown TLS mode '{other}', defaulting to system")),
        ),
    }
}

fn finalize_packs(
    packs_layer: PacksLayer,
    paths: &PathsConfig,
    default_index_path: &std::path::Path,
    defaults: &DefaultPaths,
) -> anyhow::Result<(PacksConfig, Vec<String>)> {
    let cache_dir = make_absolute(
        packs_layer
            .cache_dir
            .unwrap_or_else(|| paths.cache_dir.join("packs")),
        defaults,
    );

    let (source, source_warnings) =
        resolve_pack_source(packs_layer.source, default_index_path, defaults)?;

    let trust = packs_layer.trust.map(|trust_layer| PackTrustConfig {
        public_keys: trust_layer.public_keys.unwrap_or_default(),
        require_signatures: trust_layer.require_signatures.unwrap_or(false),
    });

    let packs = PacksConfig {
        source,
        cache_dir,
        index_cache_ttl_secs: packs_layer.index_cache_ttl_secs,
        trust,
    };

    Ok((packs, source_warnings))
}

fn resolve_pack_source(
    source_layer: Option<PackSourceLayer>,
    default_index_path: &std::path::Path,
    defaults: &DefaultPaths,
) -> anyhow::Result<(PackSourceConfig, Vec<String>)> {
    let source = match source_layer.unwrap_or_default() {
        PackSourceLayer::LocalIndex { path } => PackSourceConfig::LocalIndex {
            path: make_absolute(
                path.unwrap_or_else(|| default_index_path.to_path_buf()),
                defaults,
            ),
        },
        PackSourceLayer::HttpIndex { url } => {
            let url =
                url.ok_or_else(|| anyhow::anyhow!("packs.source.url is required for http_index"))?;
            PackSourceConfig::HttpIndex { url }
        }
        PackSourceLayer::OciRegistry { reference } => {
            let reference = reference.ok_or_else(|| {
                anyhow::anyhow!("packs.source.reference is required for oci_registry")
            })?;
            PackSourceConfig::OciRegistry { reference }
        }
    };
    Ok((source, Vec::new()))
}

fn default_pack_index_path(paths: &PathsConfig, defaults: &DefaultPaths) -> PathBuf {
    let state_index = paths.state_dir.join("packs").join("index.json");
    let candidate = if state_index.as_os_str().is_empty() {
        paths.greentic_root.join("packs").join("index.json")
    } else {
        state_index
    };
    make_absolute(candidate, defaults)
}
