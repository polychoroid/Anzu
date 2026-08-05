use nalgebra::Vector4;

use crate::assets::mesh_provider::Primitive;

#[derive(Debug, Clone, PartialEq)]
pub struct TextureDemoContent {
    pub geometry_primitive: Primitive,
    pub vertex_colors: &'static [[f32; 4]],
    pub texture_id: TextureDemoTextureId,
    pub initial_background_color: Vector4<f64>,
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

const PENTAGON_VERTEX_COLORS: [[f32; 4]; 5] = [
    [1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
];

pub struct TextureDemoRom {
    queued_command: Option<TextureDemoRenderCommand>,
    emitted_initial_command: bool,
}

impl TextureDemoRom {
    pub fn new() -> Self {
        Self {
            queued_command: None,
            emitted_initial_command: false,
        }
    }

    pub fn content() -> TextureDemoContent {
        TextureDemoContent {
            geometry_primitive: Primitive::Pentagon,
            vertex_colors: &PENTAGON_VERTEX_COLORS,
            texture_id: TextureDemoTextureId::BrickReclaimedRunningBaseColor,
            initial_background_color: Vector4::new(0.1, 0.2, 0.3, 1.0),
        }
    }

    pub fn resolve_texture_bytes(texture_id: TextureDemoTextureId) -> &'static [u8] {
        match texture_id {
            TextureDemoTextureId::BrickReclaimedRunningBaseColor => include_bytes!(
                "files/textures/Poliigon_BrickReclaimedRunning_7787_BaseColor.jpg"
            ),
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

        assert_eq!(startup.geometry_primitive, Primitive::Pentagon);
        assert_eq!(startup.vertex_colors.len(), 5);
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
    fn startup_content_resolves_brick_texture_bytes() {
        let startup = TextureDemoRom::startup_content();

        assert!(!startup.texture_bytes.is_empty());
        assert_eq!(startup.vertex_colors.len(), 5);
    }
}
