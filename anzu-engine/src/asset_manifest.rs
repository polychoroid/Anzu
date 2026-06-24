#[cfg(target_arch = "wasm32")]
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use serde::Deserialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Deserialize)]
pub struct AssetManifest {
    pub version: String,
    pub assets: Vec<AssetRecord>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Deserialize)]
pub struct AssetRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub source_url: String,
    #[serde(default)]
    pub hash: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct AssetRegistry {
    pub manifest: AssetManifest,
    entries_by_id: HashMap<String, AssetRecord>,
}

#[cfg(target_arch = "wasm32")]
impl AssetRegistry {
    pub fn asset_count(&self) -> usize {
        self.entries_by_id.len()
    }

    pub fn manifest_version(&self) -> &str {
        &self.manifest.version
    }

    pub fn hashed_asset_count(&self) -> usize {
        self.entries_by_id
            .values()
            .filter(|entry| {
                entry
                    .hash
                    .as_deref()
                    .is_some_and(|hash| !hash.trim().is_empty())
            })
            .count()
    }

    pub fn asset_type_breakdown(&self) -> String {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for entry in self.entries_by_id.values() {
            *counts.entry(entry.asset_type.as_str()).or_insert(0) += 1;
        }

        counts
            .into_iter()
            .map(|(asset_type, count)| format!("{asset_type}:{count}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub enum ManifestLoadError {
    BrowserUnavailable,
    FetchFailed(String),
    InvalidResponse(String),
    HttpStatus(u16),
    BodyReadFailed(String),
    InvalidBody,
    InvalidJson(String),
    EmptyAssetId,
    EmptySourceUrl(String),
    DuplicateAssetId(String),
}

#[cfg(target_arch = "wasm32")]
impl ManifestLoadError {
    pub fn into_js_value(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for ManifestLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestLoadError::BrowserUnavailable => {
                write!(
                    f,
                    "Asset manifest load failed: browser window is unavailable"
                )
            }
            ManifestLoadError::FetchFailed(error) => {
                write!(f, "Asset manifest load failed during fetch: {error}")
            }
            ManifestLoadError::InvalidResponse(error) => {
                write!(f, "Asset manifest load returned invalid response: {error}")
            }
            ManifestLoadError::HttpStatus(status) => {
                write!(f, "Asset manifest request failed with HTTP status {status}")
            }
            ManifestLoadError::BodyReadFailed(error) => {
                write!(f, "Asset manifest body read failed: {error}")
            }
            ManifestLoadError::InvalidBody => {
                write!(f, "Asset manifest response body was not valid text")
            }
            ManifestLoadError::InvalidJson(error) => {
                write!(f, "Asset manifest JSON parse failed: {error}")
            }
            ManifestLoadError::EmptyAssetId => {
                write!(f, "Asset manifest contains an asset entry with an empty id")
            }
            ManifestLoadError::EmptySourceUrl(asset_id) => {
                write!(
                    f,
                    "Asset manifest entry '{asset_id}' has an empty source_url"
                )
            }
            ManifestLoadError::DuplicateAssetId(asset_id) => {
                write!(f, "Asset manifest contains duplicate asset id '{asset_id}'")
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl std::error::Error for ManifestLoadError {}

#[cfg(target_arch = "wasm32")]
pub async fn load_from_url(url: &str) -> Result<AssetRegistry, ManifestLoadError> {
    let window = web_sys::window().ok_or(ManifestLoadError::BrowserUnavailable)?;

    let response_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|error| ManifestLoadError::FetchFailed(format!("{error:?}")))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|error| ManifestLoadError::InvalidResponse(format!("{error:?}")))?;

    if !response.ok() {
        return Err(ManifestLoadError::HttpStatus(response.status()));
    }

    let body_value = JsFuture::from(
        response
            .text()
            .map_err(|error| ManifestLoadError::BodyReadFailed(format!("{error:?}")))?,
    )
    .await
    .map_err(|error| ManifestLoadError::BodyReadFailed(format!("{error:?}")))?;

    let body = body_value
        .as_string()
        .ok_or(ManifestLoadError::InvalidBody)?;

    let manifest: AssetManifest = serde_json::from_str(&body)
        .map_err(|error| ManifestLoadError::InvalidJson(error.to_string()))?;

    let mut entries_by_id = HashMap::with_capacity(manifest.assets.len());
    for entry in &manifest.assets {
        if entry.id.trim().is_empty() {
            return Err(ManifestLoadError::EmptyAssetId);
        }

        if entry.source_url.trim().is_empty() {
            return Err(ManifestLoadError::EmptySourceUrl(entry.id.clone()));
        }

        if entries_by_id
            .insert(entry.id.clone(), entry.clone())
            .is_some()
        {
            return Err(ManifestLoadError::DuplicateAssetId(entry.id.clone()));
        }
    }

    Ok(AssetRegistry {
        manifest,
        entries_by_id,
    })
}
