use nalgebra::{Rotation3, Translation3, Vector3, Vector4};

use crate::assets::mesh_provider::Primitive;
use crate::execution::render::ProjectionMode;
use crate::simulation::physics::{BodyState, MotionVector};

#[derive(Debug, Clone, PartialEq)]
pub struct TextureDemoContent {
    pub geometry_primitive: Primitive,
    pub vertex_colors: &'static [[f32; 4]],
    pub texture_id: TextureDemoTextureId,
    pub initial_background_color: Vector4<f64>,
    pub projection_mode: ProjectionMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureDemoRenderCommand {
    SetBackgroundColor(Vector4<f64>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureDemoTextureId {
    BrickReclaimedRunningBaseColor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureDemoStartupContent {
    pub geometry_primitive: Primitive,
    pub vertex_colors: &'static [[f32; 4]],
    pub texture_bytes: &'static [u8],
}

const TETRAHEDRON_VERTEX_COLORS: [[f32; 4]; 4] = [
    [1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
];

pub struct TextureDemoRom {
    queued_command: Option<TextureDemoRenderCommand>,
    emitted_initial_command: bool,
    body_state: Option<BodyState>,
    projection_mode: ProjectionMode,
}

impl TextureDemoRom {
    pub fn new() -> Self {
        Self {
            queued_command: None,
            emitted_initial_command: false,
            projection_mode: Self::content().projection_mode,
            body_state: Some(BodyState::new_with_runtime_transform(
                1.0,
                crate::assets::MeshType::BuiltIn(Primitive::Tetrahedron),
                Some(MotionVector {
                    linear: Vector3::zeros(),
                    angular: Vector3::new(0.0, 1.0, 0.0),
                }),
                None,
                Some(Box::new(|_body, step_index| {
                    let phase = step_index as f32 * 0.06;
                    let rotation = Rotation3::from_euler_angles(0.0, phase, 0.0);
                    let translation = Translation3::new(0.0, 0.0, -2.0).to_homogeneous();
                    translation * rotation.to_homogeneous()
                })),
            )),
        }
    }

    pub fn content() -> TextureDemoContent {
        TextureDemoContent {
            geometry_primitive: Primitive::Tetrahedron,
            vertex_colors: &TETRAHEDRON_VERTEX_COLORS,
            texture_id: TextureDemoTextureId::BrickReclaimedRunningBaseColor,
            initial_background_color: Vector4::new(0.1, 0.2, 0.3, 1.0),
            projection_mode: ProjectionMode::Isometric,
        }
    }

    pub fn resolve_texture_bytes(texture_id: TextureDemoTextureId) -> &'static [u8] {
        match texture_id {
            TextureDemoTextureId::BrickReclaimedRunningBaseColor => {
                include_bytes!("files/textures/Poliigon_BrickReclaimedRunning_7787_BaseColor.jpg")
            }
        }
    }

    pub fn startup_content() -> TextureDemoStartupContent {
        let content = Self::content();
        TextureDemoStartupContent {
            geometry_primitive: content.geometry_primitive,
            vertex_colors: content.vertex_colors,
            texture_bytes: Self::resolve_texture_bytes(content.texture_id),
        }
    }

    pub fn initial_render_command() -> Option<TextureDemoRenderCommand> {
        let content = Self::content();
        Some(TextureDemoRenderCommand::SetBackgroundColor(
            content.initial_background_color,
        ))
    }

    pub fn body_state(&self) -> Option<&BodyState> {
        self.body_state.as_ref()
    }

    pub fn projection_mode(&self) -> ProjectionMode {
        self.projection_mode
    }

    pub fn toggle_projection_mode(&mut self) -> ProjectionMode {
        self.projection_mode = self.projection_mode.next();
        self.projection_mode
    }

    pub fn take_body_state(&mut self) -> BodyState {
        self.body_state
            .take()
            .expect("texture demo ROM should initialize its body state")
    }

    pub fn next_render_command(&mut self) -> Option<TextureDemoRenderCommand> {
        if !self.emitted_initial_command {
            self.emitted_initial_command = true;
            if let Some(initial) = Self::initial_render_command() {
                return Some(initial);
            }
        }

        self.queued_command.take()
    }

    pub fn queue_background_color_for_tick(&mut self, rgba: Vector4<f64>) -> bool {
        if self.queued_command.is_some() {
            return false;
        }

        self.queued_command = Some(TextureDemoRenderCommand::SetBackgroundColor(rgba));
        true
    }
}

impl Default for TextureDemoRom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_initial_command_once() {
        let mut rom = TextureDemoRom::new();

        assert_eq!(
            rom.next_render_command(),
            TextureDemoRom::initial_render_command()
        );
        assert_eq!(rom.next_render_command(), None);
    }

    #[test]
    fn rejects_second_queued_command_in_same_tick() {
        let mut rom = TextureDemoRom::new();
        let _ = rom.next_render_command();

        assert_eq!(
            rom.queue_background_color_for_tick(Vector4::new(0.2, 0.2, 0.2, 1.0)),
            true
        );
        assert_eq!(
            rom.queue_background_color_for_tick(Vector4::new(0.4, 0.4, 0.4, 1.0)),
            false
        );
        assert_eq!(
            rom.next_render_command(),
            Some(TextureDemoRenderCommand::SetBackgroundColor(Vector4::new(
                0.2, 0.2, 0.2, 1.0,
            )))
        );
    }

    #[test]
    fn exposes_initial_geometry_from_rom_content() {
        let startup = TextureDemoRom::startup_content();

        assert_eq!(startup.geometry_primitive, Primitive::Tetrahedron);
        assert_eq!(startup.vertex_colors.len(), 4);
        assert_eq!(startup.vertex_colors[0], [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn initial_render_command_uses_rom_content_background_color() {
        let content = TextureDemoRom::content();

        assert_eq!(
            TextureDemoRom::initial_render_command(),
            Some(TextureDemoRenderCommand::SetBackgroundColor(
                content.initial_background_color
            ))
        );
    }

    #[test]
    fn rom_content_specifies_isometric_projection() {
        let content = TextureDemoRom::content();

        assert_eq!(content.projection_mode, ProjectionMode::Isometric);
    }

    #[test]
    fn rom_cycles_projection_mode() {
        let mut rom = TextureDemoRom::new();

        assert_eq!(rom.projection_mode(), ProjectionMode::Isometric);
        assert_eq!(rom.toggle_projection_mode(), ProjectionMode::Perspective);
        assert_eq!(rom.toggle_projection_mode(), ProjectionMode::Isometric);
    }

    #[test]
    fn startup_content_resolves_brick_texture_bytes() {
        let startup = TextureDemoRom::startup_content();

        assert!(!startup.texture_bytes.is_empty());
        assert_eq!(startup.vertex_colors.len(), 4);
    }

    #[test]
    fn rom_initializes_body_state_with_custom_runtime_transform() {
        let rom = TextureDemoRom::new();
        let body_state = rom.body_state().expect("body state should be initialized");

        let transform = body_state.runtime_transform(2);
        let expected = nalgebra::Rotation3::from_euler_angles(0.0, 0.12, 0.0).to_homogeneous();

        assert!((transform[(0, 0)] - expected[(0, 0)]).abs() < 1e-6);
        assert!((transform[(1, 1)] - expected[(1, 1)]).abs() < 1e-6);
    }
}
