use crate::{ProvenanceMap, ProvenanceMapDetailed, ProvenanceRecord};
use greentic_config_types::{ConfigSource, GreenticConfig};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ExplainReport {
    pub text: String,
    pub json: serde_json::Value,
}

impl ExplainReport {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.text.clone()
    }

    pub fn to_json(&self) -> serde_json::Value {
        self.json.clone()
    }
}

pub fn explain(
    config: &GreenticConfig,
    provenance: &ProvenanceMap,
    warnings: &[String],
) -> ExplainReport {
    let mut lines = Vec::new();
    lines.push("Configuration:".to_string());
    lines.push(format!(
        "- schema_version: {}",
        config.schema_version.0.as_str()
    ));
    let env_prov = provenance.get(&greentic_config_types::ProvenancePath(
        "environment.env_id".into(),
    ));
    lines.push(format!(
        "- environment.env_id: {:?} ({})",
        config.environment.env_id,
        render_source(env_prov)
    ));
    lines.push(format!(
        "- paths.state_dir: {} ({})",
        config.paths.state_dir.display(),
        render_source(provenance.get(&greentic_config_types::ProvenancePath(
            "paths.state_dir".into()
        )))
    ));
    lines.push(format!(
        "- telemetry.exporter: {:?} ({})",
        config.telemetry.exporter,
        render_source(provenance.get(&greentic_config_types::ProvenancePath(
            "telemetry.exporter".into()
        )))
    ));
    lines.push(format!(
        "- network.tls_mode: {:?} ({})",
        config.network.tls_mode,
        render_source(provenance.get(&greentic_config_types::ProvenancePath(
            "network.tls_mode".into()
        )))
    ));
    if let Some(deployer) = &config.deployer
        && let Some(base_domain) = &deployer.base_domain
    {
        lines.push(format!(
            "- deployer.base_domain: {} ({})",
            base_domain,
            render_source(provenance.get(&greentic_config_types::ProvenancePath(
                "deployer.base_domain".into()
            )))
        ));
    }
    if let Some(services) = &config.services
        && let Some(events) = &services.events
    {
        lines.push(format!(
            "- services.events.url: {} ({})",
            events.url,
            render_source(provenance.get(&greentic_config_types::ProvenancePath(
                "services.events.url".into()
            )))
        ));
    }
    if let Some(runtime) = &config.runtime.admin_endpoints {
        lines.push(format!(
            "- runtime.admin_endpoints.secrets_explain_enabled: {} ({})",
            runtime.secrets_explain_enabled,
            render_source(provenance.get(&greentic_config_types::ProvenancePath(
                "runtime.admin_endpoints.secrets_explain_enabled".into()
            )))
        ));
    }
    if let Some(events) = &config.events {
        if let Some(backoff) = &events.backoff {
            lines.push(format!(
                "- events.backoff.initial_ms: {:?} ({})",
                backoff.initial_ms,
                render_source(provenance.get(&greentic_config_types::ProvenancePath(
                    "events.backoff".into()
                )))
            ));
        }
        if let Some(reconnect) = &events.reconnect {
            lines.push(format!(
                "- events.reconnect.enabled: {:?} ({})",
                reconnect.enabled,
                render_source(provenance.get(&greentic_config_types::ProvenancePath(
                    "events.reconnect".into()
                )))
            ));
        }
    }

    if !warnings.is_empty() {
        lines.push("Warnings:".into());
        for warning in warnings {
            lines.push(format!("  - {warning}"));
        }
    }

    let json = serde_json::json!({
        "config": config,
        "provenance": provenance_as_json(provenance),
        "warnings": warnings,
    });

    ExplainReport {
        text: lines.join("\n"),
        json,
    }
}

pub fn explain_detailed(
    config: &GreenticConfig,
    provenance: &ProvenanceMapDetailed,
    warnings: &[String],
) -> ExplainReport {
    let mut lines = Vec::new();
    lines.push("Configuration:".to_string());
    lines.push(format!(
        "- schema_version: {}",
        config.schema_version.0.as_str()
    ));
    lines.push(format!(
        "- environment.env_id: {:?} ({})",
        config.environment.env_id,
        render_record(provenance.get(&greentic_config_types::ProvenancePath(
            "environment.env_id".into()
        )))
    ));
    lines.push(format!(
        "- paths.state_dir: {} ({})",
        config.paths.state_dir.display(),
        render_record(provenance.get(&greentic_config_types::ProvenancePath(
            "paths.state_dir".into()
        )))
    ));
    if let Some(services) = &config.services
        && let Some(events) = &services.events
    {
        lines.push(format!(
            "- services.events.url: {} ({})",
            events.url,
            render_record(provenance.get(&greentic_config_types::ProvenancePath(
                "services.events.url".into()
            )))
        ));
    }
    if let Some(runtime) = &config.runtime.admin_endpoints {
        lines.push(format!(
            "- runtime.admin_endpoints.secrets_explain_enabled: {} ({})",
            runtime.secrets_explain_enabled,
            render_record(provenance.get(&greentic_config_types::ProvenancePath(
                "runtime.admin_endpoints.secrets_explain_enabled".into()
            )))
        ));
    }
    if !warnings.is_empty() {
        lines.push("Warnings:".into());
        for warning in warnings {
            lines.push(format!("  - {warning}"));
        }
    }

    let json = serde_json::json!({
        "config": config,
        "provenance": provenance_as_json_detailed(provenance),
        "warnings": warnings,
    });

    ExplainReport {
        text: lines.join("\n"),
        json,
    }
}

fn render_source(source: Option<&ConfigSource>) -> String {
    match source {
        Some(ConfigSource::Default) => "default".into(),
        Some(ConfigSource::User) => "user".into(),
        Some(ConfigSource::Project) => "project".into(),
        Some(ConfigSource::Environment) => "env".into(),
        Some(ConfigSource::Cli) => "cli".into(),
        None => "unknown".into(),
    }
}

fn render_record(record: Option<&ProvenanceRecord>) -> String {
    let Some(rec) = record else {
        return "unknown".into();
    };
    let source = render_source(Some(&rec.source));
    match rec.origin.as_deref() {
        Some(origin) => format!("{source}@{origin}"),
        None => source,
    }
}

fn provenance_as_json(provenance: &ProvenanceMap) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = provenance
        .iter()
        .map(|(k, v)| {
            (
                k.0.clone(),
                serde_json::Value::String(render_source(Some(v))),
            )
        })
        .collect();
    serde_json::Value::Object(map)
}

fn provenance_as_json_detailed(provenance: &ProvenanceMapDetailed) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = provenance
        .iter()
        .map(|(k, v)| {
            (
                k.0.clone(),
                serde_json::json!({"source": render_source(Some(&v.source)), "origin": v.origin}),
            )
        })
        .collect();
    serde_json::Value::Object(map)
}
