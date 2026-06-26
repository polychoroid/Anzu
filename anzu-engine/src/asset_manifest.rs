use std::collections::BTreeMap;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::simulation::PhysicsConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct AssetManifest {
    #[serde(default)]
    pub rom_id: Option<String>,
    pub version: String,
    #[serde(default)]
    pub entry_scene: Option<String>,
    pub assets: Vec<AssetRecord>,
    #[serde(default)]
    pub scenes: Vec<SceneRecord>,
    #[serde(default)]
    pub controller_maps: Vec<ControllerMapRecord>,
    #[serde(default)]
    pub entity_templates: Vec<EntityTemplateRecord>,
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub source_url: String,
    #[serde(default)]
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SceneRecord {
    pub id: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControllerMapRecord {
    pub id: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntityTemplateRecord {
    pub id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub controller_map_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SceneDefinition {
    pub scene_id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub entity_templates: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControllerBinding {
    pub action: String,
    pub input: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControllerMapDefinition {
    pub map_id: String,
    #[serde(default)]
    pub bindings: Vec<ControllerBinding>,
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

    pub fn entry_scene(&self) -> Option<&str> {
        self.manifest.entry_scene.as_deref()
    }

    pub fn scene_count(&self) -> usize {
        self.manifest.scenes.len()
    }

    pub fn controller_map_count(&self) -> usize {
        self.manifest.controller_maps.len()
    }

    pub fn entity_template_count(&self) -> usize {
        self.manifest.entity_templates.len()
    }

    pub fn controller_maps(&self) -> &[ControllerMapRecord] {
        &self.manifest.controller_maps
    }

    pub fn scene_source_url(&self, scene_id: &str) -> Option<&str> {
        self.manifest
            .scenes
            .iter()
            .find(|scene| scene.id == scene_id)
            .map(|scene| scene.source_url.as_str())
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

    pub fn physics_config(&self) -> PhysicsConfig {
        let defaults = PhysicsConfig::default();
        let Some(settings) = &self.manifest.settings else {
            return defaults;
        };
        let Some(physics) = &settings.physics else {
            return defaults;
        };

        PhysicsConfig {
            world_min_x: physics.world_min_x.unwrap_or(defaults.world_min_x),
            world_max_x: physics.world_max_x.unwrap_or(defaults.world_max_x),
            world_min_y: physics.world_min_y.unwrap_or(defaults.world_min_y),
            world_max_y: physics.world_max_y.unwrap_or(defaults.world_max_y),
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
    EmptyEntryScene,
    EmptyAssetId,
    EmptySourceUrl(String),
    EmptyAssetHash(String),
    DuplicateAssetId(String),
    EmptySceneId,
    EmptySceneSourceUrl(String),
    DuplicateSceneId(String),
    EmptyControllerMapId,
    EmptyControllerMapSourceUrl(String),
    DuplicateControllerMapId(String),
    EmptyEntityTemplateId,
    DuplicateEntityTemplateId(String),
}

impl ManifestLoadError {
    pub fn into_js_value(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
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
            ManifestLoadError::EmptyEntryScene => {
                write!(f, "Asset manifest contains an empty entry_scene")
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
            ManifestLoadError::EmptySceneId => {
                write!(f, "Asset manifest contains a scene entry with an empty id")
            }
            ManifestLoadError::EmptySceneSourceUrl(scene_id) => {
                write!(
                    f,
                    "Asset manifest scene entry '{scene_id}' has an empty source_url"
                )
            }
            ManifestLoadError::DuplicateSceneId(scene_id) => {
                write!(f, "Asset manifest contains duplicate scene id '{scene_id}'")
            }
            ManifestLoadError::EmptyControllerMapId => {
                write!(
                    f,
                    "Asset manifest contains a controller map entry with an empty id"
                )
            }
            ManifestLoadError::EmptyControllerMapSourceUrl(map_id) => {
                write!(
                    f,
                    "Asset manifest controller map entry '{map_id}' has an empty source_url"
                )
            }
            ManifestLoadError::DuplicateControllerMapId(map_id) => {
                write!(
                    f,
                    "Asset manifest contains duplicate controller map id '{map_id}'"
                )
            }
            ManifestLoadError::EmptyEntityTemplateId => {
                write!(
                    f,
                    "Asset manifest contains an entity template entry with an empty id"
                )
            }
            ManifestLoadError::DuplicateEntityTemplateId(template_id) => {
                write!(
                    f,
                    "Asset manifest contains duplicate entity template id '{template_id}'"
                )
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

    if manifest
        .entry_scene
        .as_deref()
        .is_some_and(|entry_scene| entry_scene.trim().is_empty())
    {
        return Err(ManifestLoadError::EmptyEntryScene);
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

    let mut scene_ids = HashMap::with_capacity(manifest.scenes.len());
    for scene in &manifest.scenes {
        if scene.id.trim().is_empty() {
            return Err(ManifestLoadError::EmptySceneId);
        }

        if scene.source_url.trim().is_empty() {
            return Err(ManifestLoadError::EmptySceneSourceUrl(scene.id.clone()));
        }

        if scene_ids.insert(scene.id.clone(), ()).is_some() {
            return Err(ManifestLoadError::DuplicateSceneId(scene.id.clone()));
        }
    }

    let mut controller_map_ids = HashMap::with_capacity(manifest.controller_maps.len());
    for controller_map in &manifest.controller_maps {
        if controller_map.id.trim().is_empty() {
            return Err(ManifestLoadError::EmptyControllerMapId);
        }

        if controller_map.source_url.trim().is_empty() {
            return Err(ManifestLoadError::EmptyControllerMapSourceUrl(
                controller_map.id.clone(),
            ));
        }

        if controller_map_ids
            .insert(controller_map.id.clone(), ())
            .is_some()
        {
            return Err(ManifestLoadError::DuplicateControllerMapId(
                controller_map.id.clone(),
            ));
        }
    }

    let mut entity_template_ids = HashMap::with_capacity(manifest.entity_templates.len());
    for template in &manifest.entity_templates {
        if template.id.trim().is_empty() {
            return Err(ManifestLoadError::EmptyEntityTemplateId);
        }

        if entity_template_ids
            .insert(template.id.clone(), ())
            .is_some()
        {
            return Err(ManifestLoadError::DuplicateEntityTemplateId(
                template.id.clone(),
            ));
        }
    }

    if let Some(entry_scene) = manifest.entry_scene.as_deref() {
        if !scene_ids.contains_key(entry_scene) {
            return Err(ManifestLoadError::InvalidJson(format!(
                "entry_scene '{entry_scene}' does not reference a declared scene id"
            )));
        }
    }

    for template in &manifest.entity_templates {
        if let Some(scene_id) = template.scene_id.as_deref() {
            if !scene_ids.contains_key(scene_id) {
                return Err(ManifestLoadError::InvalidJson(format!(
                    "entity template '{}' references unknown scene_id '{scene_id}'",
                    template.id
                )));
            }
        }

        if let Some(controller_map_id) = template.controller_map_id.as_deref() {
            if !controller_map_ids.contains_key(controller_map_id) {
                return Err(ManifestLoadError::InvalidJson(format!(
                    "entity template '{}' references unknown controller_map_id '{controller_map_id}'",
                    template.id
                )));
            }
        }
    }

    Ok(AssetRegistry {
        manifest,
        entries_by_id,
    })
}

pub async fn load_scene_definition(url: &str) -> Result<SceneDefinition, ManifestLoadError> {
    let body = fetch_text_from_url(url).await?;
    serde_json::from_str(&body).map_err(|error| {
        ManifestLoadError::InvalidJson(format!(
            "scene definition parse failed for '{url}': {error}"
        ))
    })
}

pub async fn load_controller_map_definition(
    url: &str,
) -> Result<ControllerMapDefinition, ManifestLoadError> {
    let body = fetch_text_from_url(url).await?;
    serde_json::from_str(&body).map_err(|error| {
        ManifestLoadError::InvalidJson(format!("controller map parse failed for '{url}': {error}"))
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
