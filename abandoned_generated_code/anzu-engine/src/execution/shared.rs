use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub type RuntimeInstant = std::time::Instant;
#[cfg(target_arch = "wasm32")]
pub type RuntimeInstant = web_time::Instant;

use nalgebra::{Matrix4, Vector4};

use crate::simulation::physics::BodyState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackgroundColorError {
    ChannelOutOfRange { channel: &'static str, value: f64 },
}

impl std::fmt::Display for BackgroundColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackgroundColorError::ChannelOutOfRange { channel, value } => {
                write!(
                    f,
                    "background color channel {channel} out of range [0.0, 1.0]: {value}"
                )
            }
        }
    }
}

impl std::error::Error for BackgroundColorError {}

impl BackgroundColor {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Result<Self, BackgroundColorError> {
        let color = Self { r, g, b, a };
        color.validate()?;
        Ok(color)
    }

    pub fn from_vector4(rgba: Vector4<f64>) -> Result<Self, BackgroundColorError> {
        Self::new(rgba.x, rgba.y, rgba.z, rgba.w)
    }

    pub fn to_vector4(self) -> Vector4<f64> {
        Vector4::new(self.r, self.g, self.b, self.a)
    }

    pub fn validate(&self) -> Result<(), BackgroundColorError> {
        validate_channel("r", self.r)?;
        validate_channel("g", self.g)?;
        validate_channel("b", self.b)?;
        validate_channel("a", self.a)?;
        Ok(())
    }
}

pub fn default_background_color() -> BackgroundColor {
    BackgroundColor {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    }
}

fn validate_channel(channel: &'static str, value: f64) -> Result<(), BackgroundColorError> {
    if (0.0..=1.0).contains(&value) {
        return Ok(());
    }

    Err(BackgroundColorError::ChannelOutOfRange { channel, value })
}

struct BodyEntry {
    state: BodyState,
    step_index: u64,
}

pub struct RuntimeState {
    scheduler: FrameScheduler,
    last_frame_at: Option<RuntimeInstant>,
    accumulated_time: Duration,
    frame_count: u64,
    background_color: BackgroundColor,
    queued_background_color: Option<BackgroundColor>,
    bodies: Vec<BodyEntry>,
}

pub struct FrameScheduler {
    fixed_step: Duration,
    max_steps: u8,
    max_frame_delta: Duration,
}

pub struct FrameStepPlan {
    pub frame_index: u64,
    pub step_count: u32,
    pub accumulated_time: Duration,
    pub remaining_time: Duration,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            scheduler: FrameScheduler::new(),
            last_frame_at: None,
            accumulated_time: Duration::ZERO,
            frame_count: 0,
            background_color: default_background_color(),
            queued_background_color: None,
            bodies: vec![BodyEntry {
                state: BodyState::default(),
                step_index: 0,
            }],
        }
    }

    pub fn set_background_color(
        &mut self,
        color: BackgroundColor,
    ) -> Result<(), BackgroundColorError> {
        color.validate()?;
        self.background_color = color;
        Ok(())
    }

    pub fn queue_background_color_update(
        &mut self,
        color: BackgroundColor,
    ) -> Result<bool, BackgroundColorError> {
        color.validate()?;
        if self.queued_background_color.is_some() {
            return Ok(false);
        }

        self.queued_background_color = Some(color);
        Ok(true)
    }

    pub fn background_color(&self) -> BackgroundColor {
        self.background_color
    }

    pub fn set_body_state(&mut self, body_state: BodyState) {
        self.set_bodies(vec![body_state]);
    }

    pub fn set_bodies(&mut self, bodies: Vec<BodyState>) {
        self.bodies = bodies
            .into_iter()
            .map(|state| BodyEntry { state, step_index: 0 })
            .collect();
    }

    pub fn body_state(&self) -> &BodyState {
        self.bodies
            .first()
            .map(|entry| &entry.state)
            .expect("runtime state should contain at least one body")
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn body_mesh_ids(&self) -> Vec<u32> {
        self.bodies.iter().map(|entry| entry.state.mesh_id()).collect()
    }

    pub fn body_transforms(&self) -> Vec<Matrix4<f32>> {
        self.bodies
            .iter()
            .map(|entry| entry.state.runtime_transform(entry.step_index))
            .collect()
    }

    pub fn transform_matrix(&self) -> Matrix4<f32> {
        self.body_transforms()
            .first()
            .copied()
            .unwrap_or_else(Matrix4::identity)
    }

    pub fn apply_animation_step(&mut self) {
        for entry in &mut self.bodies {
            entry.step_index += 1;
        }
    }

    pub fn begin_frame(&mut self, now: RuntimeInstant) -> FrameStepPlan {
        if let Some(next_color) = self.queued_background_color.take() {
            self.background_color = next_color;
        }

        let elapsed = self
            .last_frame_at
            .map(|last| now.saturating_duration_since(last))
            .unwrap_or_default();
        let clamped_elapsed = elapsed.min(self.scheduler.max_frame_delta);
        self.last_frame_at = Some(now);
        self.frame_count += 1;
        self.accumulated_time += clamped_elapsed;
        let plan = self
            .scheduler
            .plan_frame(self.frame_count, self.accumulated_time);

        for _ in 0..plan.step_count {
            self.apply_animation_step();
        }

        self.accumulated_time = plan.remaining_time;
        plan
    }
}

impl FrameScheduler {
    pub fn new() -> Self {
        Self {
            fixed_step: Duration::from_millis(16),
            max_steps: 5,
            max_frame_delta: Duration::from_millis(250),
        }
    }

    pub fn plan_frame(&mut self, frame_index: u64, accumulated_time: Duration) -> FrameStepPlan {
        let clamped_time = accumulated_time.min(self.max_frame_delta);
        let step_count = (clamped_time.as_nanos() / self.fixed_step.as_nanos()) as u32;
        let step_count = step_count.min(self.max_steps as u32);
        let remaining_time = clamped_time.saturating_sub(self.fixed_step * step_count as u32);

        FrameStepPlan {
            frame_index,
            step_count,
            accumulated_time: clamped_time,
            remaining_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{MeshType, mesh_provider::Primitive};
    use crate::simulation::physics::MotionVector;

    #[test]
    fn scheduler_plans_a_single_step_for_steady_delta() {
        let mut scheduler = FrameScheduler::new();
        let plan = scheduler.plan_frame(1, Duration::from_millis(16));

        assert_eq!(plan.step_count, 1);
        assert_eq!(plan.remaining_time, Duration::from_millis(0));
    }

    #[test]
    fn runtime_state_tracks_frame_progress() {
        let mut runtime = RuntimeState::new();
        let plan = runtime.begin_frame(RuntimeInstant::now());

        assert_eq!(plan.step_count, 0);
        assert!(runtime.frame_count > 0);
    }

    #[test]
    fn runtime_state_clamps_large_frame_deltas() {
        let mut runtime = RuntimeState::new();
        let base = RuntimeInstant::now();
        let _ = runtime.begin_frame(base);

        let plan = runtime.begin_frame(base + Duration::from_millis(500));

        assert_eq!(plan.step_count, 5);
        assert!(plan.remaining_time <= Duration::from_millis(250));
        assert!(runtime.accumulated_time <= Duration::from_millis(250));
    }

    #[test]
    fn runtime_state_carries_remainder_across_steps() {
        let mut runtime = RuntimeState::new();
        let base = RuntimeInstant::now();
        let _ = runtime.begin_frame(base);

        let plan = runtime.begin_frame(base + Duration::from_millis(33));

        assert_eq!(plan.step_count, 2);
        assert!(plan.remaining_time <= Duration::from_millis(16));
        assert!(runtime.accumulated_time <= Duration::from_millis(16));
    }

    #[test]
    fn runtime_state_does_not_advance_for_zero_delta() {
        let mut runtime = RuntimeState::new();
        let base = RuntimeInstant::now();
        let first = runtime.begin_frame(base);
        let second = runtime.begin_frame(base);

        assert_eq!(first.step_count, 0);
        assert_eq!(second.step_count, 0);
        assert_eq!(runtime.accumulated_time, Duration::ZERO);
    }

    #[test]
    fn frame_scheduler_caps_steps_for_large_accumulation() {
        let mut scheduler = FrameScheduler::new();
        scheduler.max_steps = 3;
        scheduler.max_frame_delta = Duration::from_secs(1);

        let plan = scheduler.plan_frame(1, Duration::from_millis(1000));

        assert_eq!(plan.step_count, 3);
        assert!(plan.remaining_time >= Duration::ZERO);
    }

    #[test]
    fn runtime_state_sets_multiple_bodies_and_exposes_transform_count() {
        let mut runtime = RuntimeState::new();
        let bodies = vec![
            BodyState::new(1.0, MeshType::BuiltIn(Primitive::Tetrahedron), None, None),
            BodyState::new(1.0, MeshType::BuiltIn(Primitive::Cube), None, None),
        ];

        runtime.set_bodies(bodies);

        assert_eq!(runtime.body_count(), 2);
        assert_eq!(runtime.body_transforms().len(), 2);
    }

    #[test]
    fn default_background_color_is_expected_value() {
        let color = default_background_color();

        assert_eq!(color.r, 0.1);
        assert_eq!(color.g, 0.2);
        assert_eq!(color.b, 0.3);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn background_color_validation_rejects_out_of_range_values() {
        let err = BackgroundColor::new(-0.1, 0.2, 0.3, 1.0).unwrap_err();

        assert_eq!(
            err,
            BackgroundColorError::ChannelOutOfRange {
                channel: "r",
                value: -0.1,
            }
        );
    }

    #[test]
    fn background_color_from_vector4_accepts_valid_values() {
        let color = BackgroundColor::from_vector4(Vector4::new(0.1, 0.2, 0.3, 1.0))
            .expect("vector4 color should be valid");

        assert_eq!(color, default_background_color());
    }

    #[test]
    fn background_color_to_vector4_round_trips_values() {
        let color = BackgroundColor::new(0.8, 0.2, 0.1, 1.0).expect("color should be valid");
        let vector = color.to_vector4();

        assert_eq!(vector, Vector4::new(0.8, 0.2, 0.1, 1.0));
    }

    #[test]
    fn runtime_state_applies_motion_to_transform_matrix() {
        let mut runtime = RuntimeState::new();
        runtime.set_body_state(BodyState::new_with_runtime_transform(
            1.0,
            crate::assets::MeshType::BuiltIn(crate::assets::mesh_provider::Primitive::Cube),
            Some(MotionVector {
                linear: nalgebra::Vector3::zeros(),
                angular: nalgebra::Vector3::new(0.0, 1.0, 0.0),
            }),
            None,
            Some(Box::new(|body, step_index| {
                let phase = step_index as f32 * 0.1;
                let motion = body.motion_vector();
                let rotation = nalgebra::Rotation3::from_euler_angles(0.0, motion.angular.y * phase, 0.0);
                let mut matrix = nalgebra::Matrix4::identity();
                matrix
                    .fixed_view_mut::<3, 3>(0, 0)
                    .copy_from(&rotation.to_homogeneous().fixed_view::<3, 3>(0, 0));
                matrix
            })),
        ));

        runtime.apply_animation_step();

        let transform = runtime.transform_matrix();
        let expected_rotation = nalgebra::Rotation3::from_euler_angles(0.0, 0.1, 0.0).to_homogeneous();
        assert!((transform[(0, 0)] - expected_rotation[(0, 0)]).abs() < 1e-6);
        assert!((transform[(1, 1)] - expected_rotation[(1, 1)]).abs() < 1e-6);
        assert!((transform[(0, 1)] - expected_rotation[(0, 1)]).abs() < 1e-6);
        assert!((transform[(1, 0)] - expected_rotation[(1, 0)]).abs() < 1e-6);
    }

    #[test]
    fn runtime_state_keeps_identity_when_motion_is_zero() {
        let mut runtime = RuntimeState::new();
        runtime.set_body_state(BodyState::default());

        runtime.apply_animation_step();

        let transform = runtime.transform_matrix();
        assert_eq!(transform[(0, 3)], 0.0);
        assert_eq!(transform[(1, 3)], 0.0);
    }

    #[test]
    fn queued_background_color_applies_on_next_frame_boundary() {
        let mut runtime = RuntimeState::new();
        let queued = BackgroundColor::new(0.8, 0.1, 0.2, 1.0).unwrap();
        runtime
            .queue_background_color_update(queued)
            .expect("queue should validate");

        assert_eq!(runtime.background_color(), default_background_color());

        let _ = runtime.begin_frame(RuntimeInstant::now());

        assert_eq!(runtime.background_color(), queued);
    }

    #[test]
    fn queue_rejects_second_command_before_boundary() {
        let mut runtime = RuntimeState::new();
        let first = BackgroundColor::new(0.4, 0.4, 0.4, 1.0).unwrap();
        let second = BackgroundColor::new(0.8, 0.8, 0.8, 1.0).unwrap();

        assert_eq!(
            runtime
                .queue_background_color_update(first)
                .expect("first queue should succeed"),
            true
        );
        assert_eq!(
            runtime
                .queue_background_color_update(second)
                .expect("second queue should be rejected without error"),
            false
        );
    }

    #[test]
    fn set_background_color_rejects_invalid_and_keeps_previous_value() {
        let mut runtime = RuntimeState::new();
        let valid = BackgroundColor::new(0.25, 0.5, 0.75, 1.0).unwrap();
        runtime
            .set_background_color(valid)
            .expect("valid color should be accepted");

        let invalid = BackgroundColor {
            r: 2.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let err = runtime
            .set_background_color(invalid)
            .expect_err("invalid color should fail");

        assert_eq!(
            err,
            BackgroundColorError::ChannelOutOfRange {
                channel: "r",
                value: 2.0,
            }
        );
        assert_eq!(runtime.background_color(), valid);
    }
}
