# Migration Status

- Added optional `services` and `events` sections to `GreenticConfig` (v0.4.x, schema version `"1"` remains). Services now carry non-secret service endpoints (`services.events.url`, headers for metadata). Events adds reconnect/backoff knobs with safe defaults.
- Resolver (`greentic-config`) applies defaults for events reconnect/backoff, validates offline + remote events endpoint, and enforces backoff sanity. No defaults for remote endpoints; leave unset unless explicitly configured.
- No breaking changes; existing configs remain valid. New fields are optional and can be introduced incrementally.
