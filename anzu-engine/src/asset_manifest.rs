use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use serde::Deserialize;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::simulation::PhysicsConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct AssetManifest {
    #[serde(default)]
    pub rom_id: Option<String>,
    pub version: String,
    pub assets: Vec<AssetRecord>,
    #[serde(default)]
    pub settings: Option<ManifestSettings>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ManifestSettings {
    #[serde(default)]
    pub physics: Option<PhysicsSettings>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PhysicsSettings {
    #[serde(default)]
    pub world_min_x: Option<f32>,
    #[serde(default)]
    pub world_max_x: Option<f32>,
    #[serde(default)]
    pub world_min_y: Option<f32>,
    #[serde(default)]
    pub world_max_y: Option<f32>,
    #[serde(default)]
    pub default_restitution: Option<f32>,
    #[serde(default)]
    pub default_friction: Option<f32>,
    #[serde(default)]
    pub broadphase_cell_size: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetRecord {
    pub id: String,
    pub source_url: String,
    #[serde(default)]
    pub hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssetRegistry {
    pub manifest: AssetManifest,
    entries_by_id: HashMap<String, AssetRecord>,
}

impl AssetRegistry {
    pub fn asset_count(&self) -> usize {
        self.entries_by_id.len()
    }

    pub fn manifest_version(&self) -> &str {
        &self.manifest.version
    }

    pub fn rom_id(&self) -> Option<&str> {
        self.manifest.rom_id.as_deref()
    }

    pub fn physics_config(&self) -> PhysicsConfig {
        let defaults = PhysicsConfig::default();
        let Some(settings) = &self.manifest.settings else {
            return defaults;
        };
        let Some(physics) = &settings.physics else {
            return defaults;
        };

        let configured_cell_size = physics
            .broadphase_cell_size
            .unwrap_or(defaults.broadphase_cell_size);

        PhysicsConfig {
            world_min_x: physics.world_min_x.unwrap_or(defaults.world_min_x),
            world_max_x: physics.world_max_x.unwrap_or(defaults.world_max_x),
            world_min_y: physics.world_min_y.unwrap_or(defaults.world_min_y),
            world_max_y: physics.world_max_y.unwrap_or(defaults.world_max_y),
            broadphase_cell_size: if configured_cell_size.is_finite() && configured_cell_size > 0.0
            {
                configured_cell_size
            } else {
                defaults.broadphase_cell_size
            },
            default_restitution: physics
                .default_restitution
                .unwrap_or(defaults.default_restitution)
                .clamp(0.0, 1.0),
            default_friction: physics
                .default_friction
                .unwrap_or(defaults.default_friction)
                .clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug)]
pub enum ManifestLoadError {
    BrowserUnavailable,
    FetchFailed(String),
    InvalidResponse(String),
    HttpStatus(u16),
    BodyReadFailed(String),
    InvalidBody,
    InvalidJson(String),
    EmptyRomId,
    EmptyAssetId,
    EmptySourceUrl(String),
    EmptyAssetHash(String),
    DuplicateAssetId(String),
}

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
            ManifestLoadError::EmptyRomId => {
                write!(f, "Asset manifest contains an empty rom_id")
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
            ManifestLoadError::EmptyAssetHash(asset_id) => {
                write!(f, "Asset manifest entry '{asset_id}' has an empty hash")
            }
            ManifestLoadError::DuplicateAssetId(asset_id) => {
                write!(f, "Asset manifest contains duplicate asset id '{asset_id}'")
            }
        }
    }
}

impl std::error::Error for ManifestLoadError {}

pub trait AssetManifestLoader {
    fn load_manifest<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AssetRegistry, ManifestLoadError>> + 'a>>;
}

pub struct WebAssetManifestLoader;

impl AssetManifestLoader for WebAssetManifestLoader {
    fn load_manifest<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AssetRegistry, ManifestLoadError>> + 'a>> {
        Box::pin(load_from_url(url))
    }
}

pub async fn load_from_url(url: &str) -> Result<AssetRegistry, ManifestLoadError> {
    let body = fetch_text_from_url(url).await?;

    let manifest: AssetManifest = serde_json::from_str(&body)
        .map_err(|error| ManifestLoadError::InvalidJson(error.to_string()))?;

    if manifest
        .rom_id
        .as_deref()
        .is_some_and(|rom_id| rom_id.trim().is_empty())
    {
        return Err(ManifestLoadError::EmptyRomId);
    }

    let mut entries_by_id = HashMap::with_capacity(manifest.assets.len());
    for entry in &manifest.assets {
        if entry.id.trim().is_empty() {
            return Err(ManifestLoadError::EmptyAssetId);
        }

        if entry.source_url.trim().is_empty() {
            return Err(ManifestLoadError::EmptySourceUrl(entry.id.clone()));
        }

        if entry
            .hash
            .as_deref()
            .map(|hash| hash.trim().is_empty())
            .unwrap_or(true)
        {
            return Err(ManifestLoadError::EmptyAssetHash(entry.id.clone()));
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

async fn fetch_text_from_url(url: &str) -> Result<String, ManifestLoadError> {
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

    body_value.as_string().ok_or(ManifestLoadError::InvalidBody)
}
