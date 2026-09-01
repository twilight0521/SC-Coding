use crate::db::Database;
use crate::providers::{
    LLMProviderAdapter, OpenAICompatibleAdapter, ProviderError, ProviderRegistry,
};
use crate::security::SecretStore;
use rusqlite::params;
use std::sync::Arc;

/// Initialize the ProviderRegistry by loading all enabled providers from the database
pub fn initialize_provider_registry(
    registry: &ProviderRegistry,
    db: &Database,
    secret_store: &SecretStore,
) -> Result<(), String> {
    let conn = db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT id, base_url, api_key_ref, default_model_id, provider_type, protocol
             FROM provider_configs
             WHERE is_enabled = 1",
        )
        .map_err(|e| e.to_string())?;

    let providers: Vec<(String, String, Option<String>, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, base_url, api_key_ref, _default_model, _provider_type, _protocol) in providers {
        let api_key = if let Some(ref key_ref) = api_key_ref {
            secret_store.retrieve(key_ref).ok()
        } else {
            None
        };

        let adapter: Arc<dyn LLMProviderAdapter> =
            Arc::new(OpenAICompatibleAdapter::new(id.clone(), base_url, api_key));

        registry.register(id, adapter);
    }

    Ok(())
}

/// Update a single provider adapter in the registry
pub fn update_provider_adapter(
    registry: &ProviderRegistry,
    provider_id: &str,
    base_url: String,
    api_key: Option<String>,
) {
    let adapter: Arc<dyn LLMProviderAdapter> = Arc::new(OpenAICompatibleAdapter::new(
        provider_id.to_string(),
        base_url,
        api_key,
    ));

    registry.register(provider_id.to_string(), adapter);
}

/// Remove a provider adapter from the registry
pub fn remove_provider_adapter(registry: &ProviderRegistry, provider_id: &str) {
    registry.remove(provider_id);
}
