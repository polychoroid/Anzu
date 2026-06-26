use std::collections::BTreeMap;

pub type EntityId = u32;

pub trait Component: 'static {}

impl<T: 'static> Component for T {}

#[derive(Clone, Copy, Default)]
pub struct Transform {
    pub position_x: f32,
    pub position_y: f32,
    pub rotation_rad: f32,
    pub uniform_scale: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub angular_velocity: f32,
    pub z_depth: f32,
}

#[derive(Clone, Copy, Default)]
pub struct Mesh {
    pub vertex_bytes: &'static [u8],
    pub vertex_count: u32,
    pub index_bytes: &'static [u8],
    pub index_count: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MeshAssetId(pub u16);

#[derive(Clone, Copy, Default)]
pub struct MeshInstance {
    pub asset_id: MeshAssetId,
}

#[derive(Clone, Copy, Default)]
pub struct CollisionBounds {
    pub radius: f32,
    pub proximity_radius: f32,
}

#[derive(Clone, Copy, Default)]
pub struct PolygonCollider {
    pub local_vertices: &'static [[f32; 2]],
}

#[derive(Clone, Copy)]
pub struct RigidBody {
    pub mass: f32,
    pub inverse_mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub moment_of_inertia: f32,
    pub inverse_moment_of_inertia: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            inverse_mass: 1.0,
            restitution: 0.82,
            friction: 0.12,
            moment_of_inertia: 1.0,
            inverse_moment_of_inertia: 1.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Lifecycle {
    pub ttl_seconds: f32,
}

pub struct SparseStorage<C: Component> {
    entries: BTreeMap<EntityId, C>,
}

impl<C: Component> Default for SparseStorage<C> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl<C: Component> SparseStorage<C> {
    pub fn insert(&mut self, entity_id: EntityId, component: C) {
        self.entries.insert(entity_id, component);
    }

    pub fn get(&self, entity_id: EntityId) -> Option<&C> {
        self.entries.get(&entity_id)
    }

    pub fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut C> {
        self.entries.get_mut(&entity_id)
    }

    pub fn remove(&mut self, entity_id: EntityId) {
        self.entries.remove(&entity_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &C)> {
        self.entries
            .iter()
            .map(|(entity_id, component)| (*entity_id, component))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut C)> {
        self.entries
            .iter_mut()
            .map(|(entity_id, component)| (*entity_id, component))
    }
}

pub struct World {
    next_entity_id: EntityId,
    transforms: SparseStorage<Transform>,
    meshes: SparseStorage<Mesh>,
    mesh_instances: SparseStorage<MeshInstance>,
    mesh_assets: BTreeMap<MeshAssetId, Mesh>,
    collision_bounds: SparseStorage<CollisionBounds>,
    polygon_colliders: SparseStorage<PolygonCollider>,
    rigid_bodies: SparseStorage<RigidBody>,
    lifecycles: SparseStorage<Lifecycle>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 1,
            transforms: SparseStorage::default(),
            meshes: SparseStorage::default(),
            mesh_instances: SparseStorage::default(),
            mesh_assets: BTreeMap::new(),
            collision_bounds: SparseStorage::default(),
            polygon_colliders: SparseStorage::default(),
            rigid_bodies: SparseStorage::default(),
            lifecycles: SparseStorage::default(),
        }
    }

    pub fn spawn(&mut self) -> EntityId {
        let entity_id = self.next_entity_id;
        self.next_entity_id = self.next_entity_id.saturating_add(1);
        entity_id
    }

    pub fn despawn(&mut self, entity_id: EntityId) {
        self.transforms.remove(entity_id);
        self.meshes.remove(entity_id);
        self.mesh_instances.remove(entity_id);
        self.collision_bounds.remove(entity_id);
        self.polygon_colliders.remove(entity_id);
        self.rigid_bodies.remove(entity_id);
        self.lifecycles.remove(entity_id);
    }

    pub fn set_transform(&mut self, entity_id: EntityId, transform: Transform) {
        self.transforms.insert(entity_id, transform);
    }

    pub fn transform(&self, entity_id: EntityId) -> Option<&Transform> {
        self.transforms.get(entity_id)
    }

    pub fn transform_mut(&mut self, entity_id: EntityId) -> Option<&mut Transform> {
        self.transforms.get_mut(entity_id)
    }

    pub fn transforms(&self) -> impl Iterator<Item = (EntityId, &Transform)> {
        self.transforms.iter()
    }

    pub fn transforms_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut Transform)> {
        self.transforms.iter_mut()
    }

    pub fn set_mesh(&mut self, entity_id: EntityId, mesh: Mesh) {
        self.mesh_instances.remove(entity_id);
        self.meshes.insert(entity_id, mesh);
    }

    pub fn register_mesh_asset(&mut self, asset_id: MeshAssetId, mesh: Mesh) {
        self.mesh_assets.insert(asset_id, mesh);
    }

    pub fn set_mesh_instance(&mut self, entity_id: EntityId, asset_id: MeshAssetId) {
        self.meshes.remove(entity_id);
        self.mesh_instances
            .insert(entity_id, MeshInstance { asset_id });
    }

    pub fn mesh(&self, entity_id: EntityId) -> Option<&Mesh> {
        self.meshes.get(entity_id)
    }

    pub fn mesh_mut(&mut self, entity_id: EntityId) -> Option<&mut Mesh> {
        self.meshes.get_mut(entity_id)
    }

    pub fn meshes(&self) -> impl Iterator<Item = (EntityId, &Mesh)> {
        self.meshes.iter()
    }

    pub fn meshes_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut Mesh)> {
        self.meshes.iter_mut()
    }

    pub fn for_each_render_mesh<F>(&self, mut callback: F)
    where
        F: FnMut(EntityId, &Mesh),
    {
        for (entity_id, instance) in self.mesh_instances.iter() {
            if let Some(mesh) = self.mesh_assets.get(&instance.asset_id) {
                callback(entity_id, mesh);
            }
        }

        for (entity_id, mesh) in self.meshes.iter() {
            if self.mesh_instances.get(entity_id).is_none() {
                callback(entity_id, mesh);
            }
        }
    }

    pub fn set_collision_bounds(&mut self, entity_id: EntityId, bounds: CollisionBounds) {
        self.collision_bounds.insert(entity_id, bounds);
    }

    pub fn collision_bounds(&self, entity_id: EntityId) -> Option<&CollisionBounds> {
        self.collision_bounds.get(entity_id)
    }

    pub fn collision_bounds_mut(&mut self, entity_id: EntityId) -> Option<&mut CollisionBounds> {
        self.collision_bounds.get_mut(entity_id)
    }

    pub fn collision_bounds_iter(&self) -> impl Iterator<Item = (EntityId, &CollisionBounds)> {
        self.collision_bounds.iter()
    }

    pub fn set_polygon_collider(&mut self, entity_id: EntityId, collider: PolygonCollider) {
        self.polygon_colliders.insert(entity_id, collider);
    }

    pub fn polygon_collider(&self, entity_id: EntityId) -> Option<&PolygonCollider> {
        self.polygon_colliders.get(entity_id)
    }

    pub fn polygon_colliders_iter(&self) -> impl Iterator<Item = (EntityId, &PolygonCollider)> {
        self.polygon_colliders.iter()
    }

    pub fn set_rigid_body(&mut self, entity_id: EntityId, rigid_body: RigidBody) {
        self.rigid_bodies.insert(entity_id, rigid_body);
    }

    pub fn rigid_body(&self, entity_id: EntityId) -> Option<&RigidBody> {
        self.rigid_bodies.get(entity_id)
    }

    pub fn rigid_body_mut(&mut self, entity_id: EntityId) -> Option<&mut RigidBody> {
        self.rigid_bodies.get_mut(entity_id)
    }

    pub fn rigid_bodies_iter(&self) -> impl Iterator<Item = (EntityId, &RigidBody)> {
        self.rigid_bodies.iter()
    }

    pub fn set_lifecycle(&mut self, entity_id: EntityId, lifecycle: Lifecycle) {
        self.lifecycles.insert(entity_id, lifecycle);
    }

    pub fn lifecycle(&self, entity_id: EntityId) -> Option<&Lifecycle> {
        self.lifecycles.get(entity_id)
    }

    pub fn lifecycle_mut(&mut self, entity_id: EntityId) -> Option<&mut Lifecycle> {
        self.lifecycles.get_mut(entity_id)
    }

    pub fn lifecycles_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut Lifecycle)> {
        self.lifecycles.iter_mut()
    }
}
