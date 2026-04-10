use crate::requests::web_requests::{get, post};

use dioxus::logger::tracing;

pub async fn get_config() -> Result<libcommon::Config, String> {
    get("/internal/config")
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch config: {e:?}");
            format!("Failed to fetch config: {e}")
        })?
        .json::<libcommon::Config>()
        .await
        .map_err(|e| {
            tracing::error!("Failed to parse config: {e:?}");
            format!("Failed to parse config: {e}")
        })
}

pub async fn get_history() -> Result<Vec<libcommon::RequestRecord>, String> {
    get("/internal/history")
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch history: {e:?}");
            format!("Failed to fetch history: {e}")
        })?
        .json::<Vec<libcommon::RequestRecord>>()
        .await
        .map_err(|e| {
            tracing::error!("Failed to parse history: {e:?}");
            format!("Failed to parse history: {e}")
        })
}

pub async fn get_mocks() -> Result<Vec<libcommon::MockConfig>, String> {
    get("/internal/mocks")
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch mocks: {e:?}");
            format!("Failed to fetch mocks: {e}")
        })?
        .json::<Vec<libcommon::MockConfig>>()
        .await
        .map_err(|e| {
            tracing::error!("Failed to parse mocks: {e:?}");
            format!("Failed to parse mocks: {e}")
        })
}

pub async fn set_mock(mock: libcommon::MockConfig) -> Result<(), String> {
    let body = serde_json::to_value(&mock).map_err(|e| {
        tracing::error!("Failed to serialize mock: {e:?}");
        format!("Failed to serialize mock: {e}")
    })?;
    post("/internal/mock", body)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set mock: {e:?}");
            format!("Failed to set mock: {e}")
        })?
        .error_for_status()
        .map_err(|e| {
            tracing::error!("Server rejected mock: {e:?}");
            format!("Server rejected mock: {e}")
        })?;
    Ok(())
}
