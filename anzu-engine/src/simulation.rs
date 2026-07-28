use std::collections::{BTreeMap, BTreeSet};
use web_time::Instant;

use crate::ecs::{EntityId, World};
use crate::input::InputEvent;

#[derive(Clone, Copy, Default)]
pub struct SimulationTransform2D {
    pub position_x: f32,
    pub position_y: f32,
    pub rotation_rad: f32,
    pub uniform_scale: f32,
}

pub trait SimulationModel {
    fn handle_input_event(&mut self, _event: InputEvent) {}
    fn sync_anchor_from_world(&mut self, _world: &World, _anchor_entity: EntityId) {}
    fn write_anchor_to_world(&self, _world: &mut World, _anchor_entity: EntityId) {}
    fn update(&mut self, delta_seconds: f32);
    fn transform_2d(&self) -> SimulationTransform2D;
    fn reconcile_world(
        &mut self,
        _world: &mut World,
        _delta_seconds: f32,
        _frame_report: &SchedulerFrameReport,
    ) {
    }
}

#[derive(Clone, Copy)]
pub struct PhysicsConfig {
    pub world_min_x: f32,
    pub world_max_x: f32,
    pub world_min_y: f32,
    pub world_max_y: f32,
    pub broadphase_cell_size: f32,
    pub default_restitution: f32,
    pub default_friction: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            world_min_x: -1.0,
            world_max_x: 1.0,
            world_min_y: -1.0,
            world_max_y: 1.0,
            broadphase_cell_size: 0.5,
            default_restitution: 0.72,
            default_friction: 0.22,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractionEventKind {
    Collision,
    ProximityEnter,
    ProximityExit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InteractionEvent {
    pub entity_a: EntityId,
    pub entity_b: EntityId,
    pub kind: InteractionEventKind,
}

#[derive(Default)]
pub struct SchedulerFrameReport {
    pub interaction_events: Vec<InteractionEvent>,
    pub despawned_entities: Vec<EntityId>,
    pub interaction_pairs_checked: u32,
    pub body_count: u32,
    pub candidate_pair_count: u32,
    pub narrowphase_checks: u32,
    pub collisions_resolved: u32,
    pub proximity_enter_count: u32,
    pub proximity_exit_count: u32,
    pub sync_ns: u64,
    pub compose_ns: u64,
    pub broadphase_ns: u64,
    pub narrowphase_ns: u64,
    pub resolve_ns: u64,
    pub reconcile_ns: u64,
    pub publish_ns: u64,
}

#[derive(Clone, Copy)]
struct BodyState {
    entity_id: EntityId,
    position_x: f32,
    position_y: f32,
    rotation_rad: f32,
    uniform_scale: f32,
    velocity_x: f32,
    velocity_y: f32,
    angular_velocity: f32,
    proximity_radius: f32,
    inverse_mass: f32,
    restitution: f32,
    friction: f32,
    inverse_moment_of_inertia: f32,
    is_sensor: bool,
    polygon_vertices: &'static [[f32; 2]],
}

#[derive(Clone, Copy)]
struct CollisionManifold {
    normal_x: f32,
    normal_y: f32,
    penetration: f32,
    contact_x: f32,
    contact_y: f32,
}

pub struct Scheduler {
    previous_proximity_pairs: BTreeSet<(EntityId, EntityId)>,
    physics_config: PhysicsConfig,
}

const BROADPHASE_GRID_SWITCH_BODY_COUNT: usize = 128;

impl Scheduler {
    pub fn new(physics_config: PhysicsConfig) -> Self {
        Self {
            previous_proximity_pairs: BTreeSet::new(),
            physics_config,
        }
    }

    pub fn update_world(&mut self, world: &mut World, delta_seconds: f32) -> SchedulerFrameReport {
        let mut report = SchedulerFrameReport::default();

        if delta_seconds <= 0.0 {
            return report;
        }

        let compose_stage_start = Instant::now();
        for (_, transform) in world.transforms_mut() {
            transform.position_x += transform.velocity_x * delta_seconds;
            transform.position_y += transform.velocity_y * delta_seconds;
            transform.rotation_rad += transform.angular_velocity * delta_seconds;
        }
        report.compose_ns = elapsed_ns_nonzero(compose_stage_start);

        let sync_stage_start = Instant::now();
        let mut entities: Vec<BodyState> = Vec::new();
        for (entity_id, collider) in world.polygon_colliders_iter() {
            if collider.local_vertices.len() < 3 {
                continue;
            }
            let Some(transform) = world.transform(entity_id).copied() else {
                continue;
            };
            let rigid_body = world.rigid_body(entity_id).copied().unwrap_or_default();
            let bounds = world
                .collision_bounds(entity_id)
                .copied()
                .unwrap_or_default();
            let proximity_radius = if bounds.proximity_radius > 0.0 {
                bounds.proximity_radius
            } else {
                max_radius(collider.local_vertices) * transform.uniform_scale.abs() * 1.5
            };

            entities.push(BodyState {
                entity_id,
                position_x: transform.position_x,
                position_y: transform.position_y,
                rotation_rad: transform.rotation_rad,
                uniform_scale: transform.uniform_scale,
                velocity_x: transform.velocity_x,
                velocity_y: transform.velocity_y,
                angular_velocity: transform.angular_velocity,
                proximity_radius,
                inverse_mass: rigid_body.inverse_mass.max(0.0),
                restitution: if rigid_body.restitution > 0.0 {
                    rigid_body.restitution.clamp(0.0, 1.0)
                } else {
                    self.physics_config.default_restitution.clamp(0.0, 1.0)
                },
                friction: if rigid_body.friction > 0.0 {
                    rigid_body.friction.clamp(0.0, 1.0)
                } else {
                    self.physics_config.default_friction.clamp(0.0, 1.0)
                },
                inverse_moment_of_inertia: rigid_body.inverse_moment_of_inertia.max(0.0),
                is_sensor: rigid_body.is_sensor,
                polygon_vertices: collider.local_vertices,
            });
        }

        let mut out_of_bounds_entities: Vec<EntityId> = Vec::new();
        entities.retain(|body| {
            if is_out_of_bounds(body, &self.physics_config) {
                out_of_bounds_entities.push(body.entity_id);
                false
            } else {
                true
            }
        });
        report.body_count = saturating_u32(entities.len());
        report.sync_ns = elapsed_ns_nonzero(sync_stage_start);

        let broadphase_stage_start = Instant::now();
        let mut candidate_pairs: Vec<(usize, usize)> = Vec::new();

        if entities.len() <= BROADPHASE_GRID_SWITCH_BODY_COUNT {
            for a in 0..entities.len() {
                for b in (a + 1)..entities.len() {
                    candidate_pairs.push((a, b));
                }
            }
        } else {
            let cell_size = sanitize_cell_size(self.physics_config.broadphase_cell_size);
            let mut broadphase_cells: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
            for (index, body) in entities.iter().enumerate() {
                let (min_cell_x, max_cell_x, min_cell_y, max_cell_y) =
                    broadphase_cell_bounds(body, cell_size);
                for cell_y in min_cell_y..=max_cell_y {
                    for cell_x in min_cell_x..=max_cell_x {
                        broadphase_cells
                            .entry((cell_x, cell_y))
                            .or_default()
                            .push(index);
                    }
                }
            }

            for occupant_indices in broadphase_cells.values() {
                for a in 0..occupant_indices.len() {
                    for b in (a + 1)..occupant_indices.len() {
                        let index_a = occupant_indices[a];
                        let index_b = occupant_indices[b];
                        candidate_pairs.push((index_a.min(index_b), index_a.max(index_b)));
                    }
                }
            }
            candidate_pairs.sort_unstable();
            candidate_pairs.dedup();
        }

        report.candidate_pair_count = saturating_u32(candidate_pairs.len());
        report.broadphase_ns = elapsed_ns_nonzero(broadphase_stage_start);

        let mut current_proximity_pairs = BTreeSet::new();
        for pair in candidate_pairs {
            report.interaction_pairs_checked = report.interaction_pairs_checked.saturating_add(1);
            let index_a = pair.0;
            let index_b = pair.1;
            let Some((state_a, state_b)) = body_pair_mut(&mut entities, index_a, index_b) else {
                continue;
            };

            let canonical_pair = (
                state_a.entity_id.min(state_b.entity_id),
                state_a.entity_id.max(state_b.entity_id),
            );

            let dx = state_b.position_x - state_a.position_x;
            let dy = state_b.position_y - state_a.position_y;
            let distance_sq = dx * dx + dy * dy;
            let proximity_distance = state_a.proximity_radius + state_b.proximity_radius;
            if distance_sq <= proximity_distance * proximity_distance {
                current_proximity_pairs.insert(canonical_pair);
                if !self.previous_proximity_pairs.contains(&canonical_pair) {
                    report.interaction_events.push(InteractionEvent {
                        entity_a: canonical_pair.0,
                        entity_b: canonical_pair.1,
                        kind: InteractionEventKind::ProximityEnter,
                    });
                    report.proximity_enter_count = report.proximity_enter_count.saturating_add(1);
                }
            }

            let narrowphase_start = Instant::now();
            report.narrowphase_checks = report.narrowphase_checks.saturating_add(1);
            let Some(manifold) = sat_polygon_collision(state_a, state_b) else {
                report.narrowphase_ns = report
                    .narrowphase_ns
                    .saturating_add(elapsed_ns_nonzero(narrowphase_start));
                continue;
            };
            report.narrowphase_ns = report
                .narrowphase_ns
                .saturating_add(elapsed_ns_nonzero(narrowphase_start));

            report.interaction_events.push(InteractionEvent {
                entity_a: canonical_pair.0,
                entity_b: canonical_pair.1,
                kind: InteractionEventKind::Collision,
            });

            if state_a.is_sensor || state_b.is_sensor {
                continue;
            }

            let resolve_start = Instant::now();
            positional_correction(
                state_a,
                state_b,
                manifold.normal_x,
                manifold.normal_y,
                manifold.penetration,
            );
            apply_contact_impulse(state_a, state_b, manifold);
            report.resolve_ns = report
                .resolve_ns
                .saturating_add(elapsed_ns_nonzero(resolve_start));
            report.collisions_resolved = report.collisions_resolved.saturating_add(1);
        }

        let early_reconcile_stage_start = Instant::now();
        for pair in &self.previous_proximity_pairs {
            if !current_proximity_pairs.contains(pair) {
                report.interaction_events.push(InteractionEvent {
                    entity_a: pair.0,
                    entity_b: pair.1,
                    kind: InteractionEventKind::ProximityExit,
                });
                report.proximity_exit_count = report.proximity_exit_count.saturating_add(1);
            }
        }
        self.previous_proximity_pairs = current_proximity_pairs;
        report.reconcile_ns = report
            .reconcile_ns
            .saturating_add(elapsed_ns_nonzero(early_reconcile_stage_start));

        let publish_stage_start = Instant::now();
        for body in &entities {
            if let Some(transform) = world.transform_mut(body.entity_id) {
                transform.position_x = body.position_x;
                transform.position_y = body.position_y;
                transform.velocity_x = body.velocity_x;
                transform.velocity_y = body.velocity_y;
                transform.angular_velocity = body.angular_velocity;
            }
        }
        report.publish_ns = elapsed_ns_nonzero(publish_stage_start);

        let late_reconcile_stage_start = Instant::now();
        let mut despawn_ids: BTreeSet<EntityId> = out_of_bounds_entities.into_iter().collect();
        for (entity_id, lifecycle) in world.lifecycles_mut() {
            lifecycle.ttl_seconds -= delta_seconds;
            if lifecycle.ttl_seconds <= 0.0 {
                despawn_ids.insert(entity_id);
            }
        }

        if !despawn_ids.is_empty() {
            self.previous_proximity_pairs
                .retain(|(a, b)| !despawn_ids.contains(a) && !despawn_ids.contains(b));
        }

        for entity_id in despawn_ids {
            world.despawn(entity_id);
            report.despawned_entities.push(entity_id);
        }
        report.reconcile_ns = report
            .reconcile_ns
            .saturating_add(elapsed_ns_nonzero(late_reconcile_stage_start));

        report
    }
}

fn elapsed_ns_nonzero(start: Instant) -> u64 {
    let elapsed = start.elapsed().as_nanos();
    if elapsed == 0 {
        1
    } else if elapsed > u64::MAX as u128 {
        u64::MAX
    } else {
        elapsed as u64
    }
}

fn saturating_u32(value: usize) -> u32 {
    if value > u32::MAX as usize {
        u32::MAX
    } else {
        value as u32
    }
}

fn sanitize_cell_size(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        PhysicsConfig::default().broadphase_cell_size
    }
}

fn grid_cell_index(value: f32, cell_size: f32) -> i32 {
    let quantized = (value / cell_size).floor();
    if !quantized.is_finite() {
        0
    } else if quantized < i32::MIN as f32 {
        i32::MIN
    } else if quantized > i32::MAX as f32 {
        i32::MAX
    } else {
        quantized as i32
    }
}

fn broadphase_cell_bounds(state: &BodyState, cell_size: f32) -> (i32, i32, i32, i32) {
    let radius = state.proximity_radius.max(0.0);
    let min_x = state.position_x - radius;
    let max_x = state.position_x + radius;
    let min_y = state.position_y - radius;
    let max_y = state.position_y + radius;

    (
        grid_cell_index(min_x, cell_size),
        grid_cell_index(max_x, cell_size),
        grid_cell_index(min_y, cell_size),
        grid_cell_index(max_y, cell_size),
    )
}

fn body_pair_mut(
    entities: &mut [BodyState],
    index_a: usize,
    index_b: usize,
) -> Option<(&mut BodyState, &mut BodyState)> {
    if index_a == index_b {
        return None;
    }
    if index_a < index_b {
        let (left, right) = entities.split_at_mut(index_b);
        Some((&mut left[index_a], &mut right[0]))
    } else {
        let (left, right) = entities.split_at_mut(index_a);
        Some((&mut right[0], &mut left[index_b]))
    }
}

fn is_out_of_bounds(state: &BodyState, config: &PhysicsConfig) -> bool {
    let polygon = transformed_polygon(state);
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for vertex in &polygon {
        min_x = min_x.min(vertex[0]);
        max_x = max_x.max(vertex[0]);
        min_y = min_y.min(vertex[1]);
        max_y = max_y.max(vertex[1]);
    }

    let width = max_x - min_x;
    let height = max_y - min_y;
    let max_dimension = width.max(height).max(0.0);

    max_x < config.world_min_x - max_dimension
        || min_x > config.world_max_x + max_dimension
        || max_y < config.world_min_y - max_dimension
        || min_y > config.world_max_y + max_dimension
}

fn positional_correction(
    state_a: &mut BodyState,
    state_b: &mut BodyState,
    nx: f32,
    ny: f32,
    penetration: f32,
) {
    let inv_mass_sum = state_a.inverse_mass + state_b.inverse_mass;
    if inv_mass_sum <= 1e-6 {
        return;
    }
    let correction = (penetration * 0.8) / inv_mass_sum;
    state_a.position_x -= nx * correction * state_a.inverse_mass;
    state_a.position_y -= ny * correction * state_a.inverse_mass;
    state_b.position_x += nx * correction * state_b.inverse_mass;
    state_b.position_y += ny * correction * state_b.inverse_mass;
}

fn apply_contact_impulse(
    state_a: &mut BodyState,
    state_b: &mut BodyState,
    manifold: CollisionManifold,
) {
    let nx = manifold.normal_x;
    let ny = manifold.normal_y;

    let rax = manifold.contact_x - state_a.position_x;
    let ray = manifold.contact_y - state_a.position_y;
    let rbx = manifold.contact_x - state_b.position_x;
    let rby = manifold.contact_y - state_b.position_y;

    let vel_a_x = state_a.velocity_x - state_a.angular_velocity * ray;
    let vel_a_y = state_a.velocity_y + state_a.angular_velocity * rax;
    let vel_b_x = state_b.velocity_x - state_b.angular_velocity * rby;
    let vel_b_y = state_b.velocity_y + state_b.angular_velocity * rbx;

    let rvx = vel_b_x - vel_a_x;
    let rvy = vel_b_y - vel_a_y;
    let vel_along_normal = rvx * nx + rvy * ny;
    if !vel_along_normal.is_finite() {
        return;
    }
    if vel_along_normal >= 0.0 {
        return;
    }

    let ra_cross_n = rax * ny - ray * nx;
    let rb_cross_n = rbx * ny - rby * nx;

    let inv_mass_sum = state_a.inverse_mass
        + state_b.inverse_mass
        + ra_cross_n * ra_cross_n * state_a.inverse_moment_of_inertia
        + rb_cross_n * rb_cross_n * state_b.inverse_moment_of_inertia;
    if !inv_mass_sum.is_finite() {
        return;
    }
    if inv_mass_sum <= 1e-6 {
        return;
    }

    let restitution = state_a.restitution.min(state_b.restitution);
    let jn = -(1.0 + restitution) * vel_along_normal / inv_mass_sum;
    let impulse_x = jn * nx;
    let impulse_y = jn * ny;

    state_a.velocity_x -= impulse_x * state_a.inverse_mass;
    state_a.velocity_y -= impulse_y * state_a.inverse_mass;
    state_b.velocity_x += impulse_x * state_b.inverse_mass;
    state_b.velocity_y += impulse_y * state_b.inverse_mass;

    state_a.angular_velocity -=
        (rax * impulse_y - ray * impulse_x) * state_a.inverse_moment_of_inertia;
    state_b.angular_velocity +=
        (rbx * impulse_y - rby * impulse_x) * state_b.inverse_moment_of_inertia;

    let tangent_x_raw = rvx - vel_along_normal * nx;
    let tangent_y_raw = rvy - vel_along_normal * ny;
    let tangent_len_sq = tangent_x_raw * tangent_x_raw + tangent_y_raw * tangent_y_raw;
    if !tangent_len_sq.is_finite() {
        return;
    }
    if tangent_len_sq <= 1e-8 {
        return;
    }

    let tangent_len = tangent_len_sq.sqrt();
    let tx = tangent_x_raw / tangent_len;
    let ty = tangent_y_raw / tangent_len;

    let vel_tangent = rvx * tx + rvy * ty;
    let ra_cross_t = rax * ty - ray * tx;
    let rb_cross_t = rbx * ty - rby * tx;
    let inv_mass_t = state_a.inverse_mass
        + state_b.inverse_mass
        + ra_cross_t * ra_cross_t * state_a.inverse_moment_of_inertia
        + rb_cross_t * rb_cross_t * state_b.inverse_moment_of_inertia;
    if !inv_mass_t.is_finite() {
        return;
    }
    if inv_mass_t <= 1e-6 {
        return;
    }

    let jt_unclamped = -vel_tangent / inv_mass_t;
    let mu = (state_a.friction * state_b.friction).sqrt();
    if !jt_unclamped.is_finite() || !mu.is_finite() || !jn.is_finite() {
        return;
    }
    let min_jt = -jn * mu;
    let max_jt = jn * mu;
    if !min_jt.is_finite() || !max_jt.is_finite() {
        return;
    }
    let jt = if min_jt <= max_jt {
        jt_unclamped.clamp(min_jt, max_jt)
    } else {
        jt_unclamped.clamp(max_jt, min_jt)
    };
    let friction_x = jt * tx;
    let friction_y = jt * ty;

    state_a.velocity_x -= friction_x * state_a.inverse_mass;
    state_a.velocity_y -= friction_y * state_a.inverse_mass;
    state_b.velocity_x += friction_x * state_b.inverse_mass;
    state_b.velocity_y += friction_y * state_b.inverse_mass;

    state_a.angular_velocity -=
        (rax * friction_y - ray * friction_x) * state_a.inverse_moment_of_inertia;
    state_b.angular_velocity +=
        (rbx * friction_y - rby * friction_x) * state_b.inverse_moment_of_inertia;
}

fn transformed_polygon(state: &BodyState) -> Vec<[f32; 2]> {
    let c = state.rotation_rad.cos();
    let s = state.rotation_rad.sin();
    let scale = state.uniform_scale;

    state
        .polygon_vertices
        .iter()
        .map(|vertex| {
            let x = vertex[0] * scale;
            let y = vertex[1] * scale;
            [
                c * x - s * y + state.position_x,
                s * x + c * y + state.position_y,
            ]
        })
        .collect()
}

fn project_on_axis(vertices: &[[f32; 2]], axis_x: f32, axis_y: f32) -> (f32, f32) {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for vertex in vertices {
        let projection = vertex[0] * axis_x + vertex[1] * axis_y;
        min = min.min(projection);
        max = max.max(projection);
    }
    (min, max)
}

fn sat_polygon_collision(state_a: &BodyState, state_b: &BodyState) -> Option<CollisionManifold> {
    if state_a.polygon_vertices.len() < 3 || state_b.polygon_vertices.len() < 3 {
        return None;
    }

    let poly_a = transformed_polygon(state_a);
    let poly_b = transformed_polygon(state_b);

    let mut min_overlap = f32::INFINITY;
    let mut best_axis = [0.0f32, 0.0f32];

    for polygon in [&poly_a, &poly_b] {
        for i in 0..polygon.len() {
            let current = polygon[i];
            let next = polygon[(i + 1) % polygon.len()];
            let edge_x = next[0] - current[0];
            let edge_y = next[1] - current[1];

            let mut axis_x = -edge_y;
            let mut axis_y = edge_x;
            let len_sq = axis_x * axis_x + axis_y * axis_y;
            if len_sq <= 1e-10 {
                continue;
            }
            let inv_len = 1.0 / len_sq.sqrt();
            axis_x *= inv_len;
            axis_y *= inv_len;

            let (min_a, max_a) = project_on_axis(&poly_a, axis_x, axis_y);
            let (min_b, max_b) = project_on_axis(&poly_b, axis_x, axis_y);
            let overlap = (max_a.min(max_b) - min_a.max(min_b)).max(0.0);
            if overlap <= 0.0 {
                return None;
            }

            if overlap < min_overlap {
                min_overlap = overlap;
                best_axis = [axis_x, axis_y];
            }
        }
    }

    let center_dx = state_b.position_x - state_a.position_x;
    let center_dy = state_b.position_y - state_a.position_y;
    if center_dx * best_axis[0] + center_dy * best_axis[1] < 0.0 {
        best_axis[0] = -best_axis[0];
        best_axis[1] = -best_axis[1];
    }

    let contact_a = support_point(&poly_a, best_axis[0], best_axis[1]);
    let contact_b = support_point(&poly_b, -best_axis[0], -best_axis[1]);

    Some(CollisionManifold {
        normal_x: best_axis[0],
        normal_y: best_axis[1],
        penetration: min_overlap,
        contact_x: (contact_a[0] + contact_b[0]) * 0.5,
        contact_y: (contact_a[1] + contact_b[1]) * 0.5,
    })
}

fn support_point(vertices: &[[f32; 2]], axis_x: f32, axis_y: f32) -> [f32; 2] {
    let mut best = vertices[0];
    let mut best_projection = best[0] * axis_x + best[1] * axis_y;
    for vertex in vertices.iter().skip(1) {
        let projection = vertex[0] * axis_x + vertex[1] * axis_y;
        if projection > best_projection {
            best_projection = projection;
            best = *vertex;
        }
    }
    best
}

fn max_radius(vertices: &[[f32; 2]]) -> f32 {
    let mut radius_sq: f32 = 0.0;
    for vertex in vertices {
        let value = vertex[0] * vertex[0] + vertex[1] * vertex[1];
        radius_sq = radius_sq.max(value);
    }
    radius_sq.sqrt()
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(PhysicsConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{InteractionEventKind, PhysicsConfig, Scheduler};
    use crate::ecs::{CollisionBounds, Lifecycle, PolygonCollider, RigidBody, Transform, World};

    const TRIANGLE_POLY: [[f32; 2]; 3] = [[-0.5, -0.5], [0.5, -0.5], [0.0, 0.6]];

    #[test]
    fn deterministic_interaction_event_order_is_stable() {
        let mut world = World::new();
        let entity_a = world.spawn();
        let entity_b = world.spawn();

        world.set_transform(
            entity_a,
            Transform {
                position_x: -0.2,
                position_y: 0.0,
                velocity_x: 0.1,
                velocity_y: 0.0,
                ..Default::default()
            },
        );
        world.set_transform(
            entity_b,
            Transform {
                position_x: 0.2,
                position_y: 0.0,
                velocity_x: -0.1,
                velocity_y: 0.0,
                ..Default::default()
            },
        );

        world.set_polygon_collider(
            entity_a,
            PolygonCollider {
                local_vertices: &TRIANGLE_POLY,
            },
        );
        world.set_polygon_collider(
            entity_b,
            PolygonCollider {
                local_vertices: &TRIANGLE_POLY,
            },
        );

        world.set_collision_bounds(
            entity_a,
            CollisionBounds {
                proximity_radius: 0.8,
            },
        );
        world.set_collision_bounds(
            entity_b,
            CollisionBounds {
                proximity_radius: 0.8,
            },
        );

        world.set_rigid_body(entity_a, RigidBody::default());
        world.set_rigid_body(entity_b, RigidBody::default());

        let mut scheduler = Scheduler::default();
        let report = scheduler.update_world(&mut world, 1.0 / 60.0);

        assert!(
            report
                .interaction_events
                .iter()
                .any(|event| event.kind == InteractionEventKind::Collision)
        );
        assert!(
            report
                .interaction_events
                .iter()
                .any(|event| event.kind == InteractionEventKind::ProximityEnter)
        );
    }

    #[test]
    fn lifecycle_despawn_removes_all_components() {
        let mut world = World::new();
        let entity = world.spawn();

        world.set_transform(entity, Transform::default());
        world.set_polygon_collider(
            entity,
            PolygonCollider {
                local_vertices: &TRIANGLE_POLY,
            },
        );
        world.set_collision_bounds(
            entity,
            CollisionBounds {
                proximity_radius: 0.8,
            },
        );
        world.set_rigid_body(entity, RigidBody::default());
        world.set_lifecycle(entity, Lifecycle { ttl_seconds: 0.01 });

        let mut scheduler = Scheduler::default();
        let report = scheduler.update_world(&mut world, 0.02);

        assert_eq!(report.despawned_entities, vec![entity]);
        assert!(world.transform(entity).is_none());
        assert!(world.polygon_collider(entity).is_none());
        assert!(world.collision_bounds(entity).is_none());
        assert!(world.rigid_body(entity).is_none());
        assert!(world.lifecycle(entity).is_none());
    }

    #[test]
    fn uniform_grid_broadphase_dedupes_and_keeps_pair_order_stable() {
        let mut world = World::new();
        let entity_a = world.spawn();
        let entity_b = world.spawn();
        let entity_c = world.spawn();

        world.set_transform(
            entity_a,
            Transform {
                position_x: -3.0,
                position_y: 0.0,
                ..Default::default()
            },
        );
        world.set_transform(
            entity_b,
            Transform {
                position_x: 0.0,
                position_y: 0.0,
                ..Default::default()
            },
        );
        world.set_transform(
            entity_c,
            Transform {
                position_x: 3.0,
                position_y: 0.0,
                ..Default::default()
            },
        );

        for entity in [entity_a, entity_b, entity_c] {
            world.set_polygon_collider(
                entity,
                PolygonCollider {
                    local_vertices: &TRIANGLE_POLY,
                },
            );
            world.set_collision_bounds(
                entity,
                CollisionBounds {
                    proximity_radius: 6.0,
                },
            );
            world.set_rigid_body(entity, RigidBody::default());
        }

        let mut scheduler = Scheduler::new(PhysicsConfig {
            world_min_x: -10.0,
            world_max_x: 10.0,
            world_min_y: -10.0,
            world_max_y: 10.0,
            broadphase_cell_size: 0.5,
            ..PhysicsConfig::default()
        });

        let report = scheduler.update_world(&mut world, 1.0 / 60.0);

        assert_eq!(report.candidate_pair_count, 3);
        assert_eq!(report.interaction_pairs_checked, 3);

        let proximity_pairs: Vec<(u32, u32)> = report
            .interaction_events
            .iter()
            .filter(|event| event.kind == InteractionEventKind::ProximityEnter)
            .map(|event| (event.entity_a, event.entity_b))
            .collect();
        assert!(!proximity_pairs.is_empty());

        let mut sorted_unique_pairs = proximity_pairs.clone();
        sorted_unique_pairs.sort_unstable();
        sorted_unique_pairs.dedup();
        assert_eq!(proximity_pairs, sorted_unique_pairs);

        for (entity_a, entity_b) in proximity_pairs {
            assert!(entity_a < entity_b);
        }
    }

    #[test]
    fn scheduler_report_includes_stage_timings_and_counters() {
        let mut world = World::new();
        let entity_a = world.spawn();
        let entity_b = world.spawn();

        world.set_transform(
            entity_a,
            Transform {
                position_x: -0.2,
                position_y: 0.0,
                velocity_x: 0.0,
                velocity_y: 0.0,
                ..Default::default()
            },
        );
        world.set_transform(
            entity_b,
            Transform {
                position_x: 0.2,
                position_y: 0.0,
                velocity_x: 0.0,
                velocity_y: 0.0,
                ..Default::default()
            },
        );

        world.set_polygon_collider(
            entity_a,
            PolygonCollider {
                local_vertices: &TRIANGLE_POLY,
            },
        );
        world.set_polygon_collider(
            entity_b,
            PolygonCollider {
                local_vertices: &TRIANGLE_POLY,
            },
        );
        world.set_collision_bounds(
            entity_a,
            CollisionBounds {
                proximity_radius: 0.8,
            },
        );
        world.set_collision_bounds(
            entity_b,
            CollisionBounds {
                proximity_radius: 0.8,
            },
        );
        world.set_rigid_body(entity_a, RigidBody::default());
        world.set_rigid_body(entity_b, RigidBody::default());

        let mut scheduler = Scheduler::default();
        let report = scheduler.update_world(&mut world, 1.0 / 60.0);

        assert!(report.sync_ns > 0);
        assert!(report.compose_ns > 0);
        assert!(report.broadphase_ns > 0);
        assert!(report.narrowphase_ns > 0);
        assert!(report.resolve_ns > 0);
        assert!(report.reconcile_ns > 0);
        assert!(report.publish_ns > 0);
        assert!(report.reconcile_ns >= 2);

        assert_eq!(report.body_count, 2);
        assert!(report.candidate_pair_count >= 1);
        assert!(report.narrowphase_checks >= 1);
    }

    #[test]
    fn repeated_runs_keep_interaction_results_deterministic() {
        fn build_world() -> World {
            let mut world = World::new();
            for y in 0..12 {
                for x in 0..12 {
                    let entity = world.spawn();
                    world.set_transform(
                        entity,
                        Transform {
                            position_x: -2.4 + x as f32 * 0.4,
                            position_y: -2.4 + y as f32 * 0.4,
                            ..Default::default()
                        },
                    );
                    world.set_polygon_collider(
                        entity,
                        PolygonCollider {
                            local_vertices: &TRIANGLE_POLY,
                        },
                    );
                    world.set_collision_bounds(
                        entity,
                        CollisionBounds {
                            proximity_radius: 0.72,
                        },
                    );
                    world.set_rigid_body(entity, RigidBody::default());
                }
            }
            world
        }

        let mut world_a = build_world();
        let mut world_b = build_world();
        let mut scheduler_a = Scheduler::new(PhysicsConfig {
            world_min_x: -10.0,
            world_max_x: 10.0,
            world_min_y: -10.0,
            world_max_y: 10.0,
            broadphase_cell_size: 0.5,
            ..PhysicsConfig::default()
        });
        let mut scheduler_b = Scheduler::new(PhysicsConfig {
            world_min_x: -10.0,
            world_max_x: 10.0,
            world_min_y: -10.0,
            world_max_y: 10.0,
            broadphase_cell_size: 0.5,
            ..PhysicsConfig::default()
        });

        let report_a = scheduler_a.update_world(&mut world_a, 1.0 / 60.0);
        let report_b = scheduler_b.update_world(&mut world_b, 1.0 / 60.0);

        assert_eq!(report_a.candidate_pair_count, report_b.candidate_pair_count);
        assert_eq!(
            report_a.interaction_pairs_checked,
            report_b.interaction_pairs_checked
        );
        assert_eq!(report_a.narrowphase_checks, report_b.narrowphase_checks);
        assert_eq!(report_a.collisions_resolved, report_b.collisions_resolved);
        assert_eq!(
            report_a.proximity_enter_count,
            report_b.proximity_enter_count
        );
        assert_eq!(report_a.proximity_exit_count, report_b.proximity_exit_count);
        assert_eq!(report_a.interaction_events, report_b.interaction_events);
    }
}
