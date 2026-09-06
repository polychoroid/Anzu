use nalgebra::{Matrix4, Rotation3, Translation3, Vector3, Vector4};

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
    bodies: Option<Vec<BodyState>>,
    projection_mode: ProjectionMode,
}

impl TextureDemoRom {
    pub fn new() -> Self {
        Self {
            queued_command: None,
            emitted_initial_command: false,
            projection_mode: Self::content().projection_mode,
            bodies: Some(Self::initial_bodies()),
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
        self.bodies.as_ref().and_then(|bodies| bodies.first())
    }

    pub fn projection_mode(&self) -> ProjectionMode {
        self.projection_mode
    }

    pub fn toggle_projection_mode(&mut self) -> ProjectionMode {
        self.projection_mode = self.projection_mode.next();
        self.projection_mode
    }

    pub fn take_bodies(&mut self) -> Vec<BodyState> {
        self.bodies
            .take()
            .expect("texture demo ROM should initialize its body states")
    }

    fn initial_bodies() -> Vec<BodyState> {
        // Scene layout: each entry is (primitive, world-space xz position).
        // The tetrahedron is centered; the others occupy the four cardinal diagonals.
        // "NW/SW" = negative-x, "NE/SE" = positive-x; "N/NE" = negative-z, "S/SW" = positive-z
        // (in view-space with the isometric camera pointing from +x+y+z toward origin).
        let spread = 1.0_f32;
        let layout: [(Primitive, f32, f32); 5] = [
            (Primitive::Tetrahedron, 0.0, 0.0),         // center
            (Primitive::Cube, -spread, -spread),        // NW
            (Primitive::Octahedron, spread, -spread),   // NE
            (Primitive::Dodecahedron, -spread, spread), // SW
            (Primitive::Icosahedron, spread, spread),   // SE
        ];

        // All bodies are normalized to fit inside this display radius.
        let target_radius = 0.35_f32;

        layout
            .into_iter()
            .map(|(primitive, tx, tz)| {
                BodyState::new_with_runtime_transform(
                    1.0,
                    crate::assets::MeshType::BuiltIn(primitive),
                    Some(MotionVector {
                        linear: Vector3::zeros(),
                        angular: Vector3::new(0.0, 1.0, 0.0),
                    }),
                    None,
                    Some(Box::new(move |body, step_index| {
                        // Scale: normalize the mesh so its longest axis fits target_radius.
                        let extents = body.extents();
                        let max_extent = extents.x.max(extents.y).max(extents.z).max(f32::EPSILON);
                        let scale = target_radius / max_extent;

                        // Alignment: rotate so the body's principal eigenvector aligns with world X.
                        // axes() columns are eigenvectors; column 0 is the principal axis.
                        let axes = body.axes();
                        let principal = axes.column(0);
                        let world_x = Vector3::x();
                        let alignment = Rotation3::rotation_between(&principal, &world_x)
                            .unwrap_or(Rotation3::identity());

                        // Animation: spin around world Y.
                        let phase = step_index as f32 * 0.04;
                        let spin = Rotation3::from_axis_angle(&Vector3::y_axis(), phase);

                        // Scene placement at depth -2.0 from the isometric camera target.
                        let translation = Translation3::new(tx, 0.0, tz - 2.0).to_homogeneous();

                        let scale_matrix = Matrix4::new_scaling(scale);
                        translation
                            * spin.to_homogeneous()
                            * alignment.to_homogeneous()
                            * scale_matrix
                    })),
                )
            })
            .collect()
    }

    /// Returns `(primitive, vertex_colors_per_vertex)` for each body, in the same order
    /// as `initial_bodies()`. The shell uses these to build per-body GPU geometry buffers.
    pub fn startup_body_specs() -> Vec<(Primitive, Vec<[f32; 4]>)> {
        [
            (Primitive::Tetrahedron, 4usize),
            (Primitive::Cube, 8),
            (Primitive::Octahedron, 6),
            (Primitive::Dodecahedron, 20),
            (Primitive::Icosahedron, 12),
        ]
        .into_iter()
        .map(|(primitive, vertex_count)| (primitive, vec![[1.0_f32, 1.0, 1.0, 1.0]; vertex_count]))
        .collect()
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
    fn rom_take_bodies_returns_expected_body_count() {
        let mut rom = TextureDemoRom::new();

        let bodies = rom.take_bodies();

        assert_eq!(bodies.len(), 5);
    }

    #[test]
    fn rom_bodies_are_the_five_platonic_solids_in_order() {
        let mut rom = TextureDemoRom::new();

        let bodies = rom.take_bodies();
        let mesh_ids: Vec<u32> = bodies.iter().map(|b| b.mesh_id()).collect();

        assert_eq!(
            mesh_ids,
            vec![
                11, // Tetrahedron
                12, // Cube
                14, // Octahedron
                16, // Dodecahedron
                15, // Icosahedron
            ],
            "bodies must be the five Platonic solids in the declared order"
        );
    }

    #[test]
    fn rom_bodies_have_distinct_phase_offsets() {
        let mut rom = TextureDemoRom::new();

        let bodies = rom.take_bodies();
        let transforms = bodies
            .iter()
            .map(|body| body.runtime_transform(2))
            .collect::<Vec<_>>();

        assert_eq!(transforms.len(), 5);
        assert!(transforms.iter().enumerate().all(|(index, transform)| {
            if index == 0 {
                true
            } else {
                (transform[(0, 0)] - transforms[0][(0, 0)]).abs() > 1e-6
                    || (transform[(1, 1)] - transforms[0][(1, 1)]).abs() > 1e-6
            }
        }));
    }

    #[test]
    fn startup_content_resolves_brick_texture_bytes() {
        let startup = TextureDemoRom::startup_content();

        assert!(!startup.texture_bytes.is_empty());
        assert_eq!(startup.vertex_colors.len(), 4);
    }

    #[test]
    fn rom_body_transform_is_non_identity_and_animates() {
        let rom = TextureDemoRom::new();
        let body_state = rom.body_state().expect("body state should be initialized");

        let t0 = body_state.runtime_transform(0);
        let t10 = body_state.runtime_transform(10);

        // Transform at step 0 should not be the identity (scale + alignment are applied).
        assert_ne!(t0, nalgebra::Matrix4::identity());
        // Transform should change as the animation advances.
        assert_ne!(t0, t10);
    }
}
