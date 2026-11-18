use anzu_runtime::assets::AssetId;
use nalgebra::{Matrix3, Vector3};
use std::collections::HashMap;
use uuid::Uuid;

pub enum Id {
    PlayerId(Uuid),
    EntityId(Uuid),
    AssetId(Uuid),
    BoatId(EntityId),
}

#[derive(Copy, Clone, Debug)]
pub struct PlayerId(Uuid);

impl PlayerId {
    pub fn new(name: &'static str) -> Self {
        Self(new_id_from_name(name))
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct EntityId(Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(new_id())
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct BoatId(EntityId);

impl BoatId {
    pub fn new(entity_id: EntityId) -> Self {
        Self(entity_id)
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum Asset {
    Boat,
    Bullet,
}

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Red,
    Blue,
}

enum Entities {
    Boat(EntityId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DamageUnit(i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ammo(i32);

pub struct PlayerStore {
    pub player_ids: [PlayerId; 2],
    pub player_colors: [Color; 2],
}

impl PlayerStore {
    pub fn new(ids: [PlayerId; 2]) -> Self {
        Self {
            player_ids: ids,
            player_colors: [Color::Red, Color::Blue],
        }
    }
}

pub struct BoatStore {
    pub index: HashMap<BoatId, usize>,
    pub entity_ids: Vec<EntityId>,
    pub player_ids: Vec<PlayerId>,
    pub asset_ids: Vec<AssetId>,
    pub colors: Vec<Color>,
    pub integrity: Vec<DamageUnit>,
    pub ammunition: Vec<Ammo>,
    pub positions: Vec<Vector3<f32>>,
    pub velocities: Vec<Vector3<f32>>,
    pub angular_velocities: Vec<Vector3<f32>>,
    pub inertia: Vec<Matrix3<f32>>,
    pub masses: Vec<f32>,
    pub boats: usize,
}

impl BoatStore {
    pub fn new(players: &PlayerStore) -> Self {
        let mut store = Self {
            index: HashMap::new(),
            entity_ids: Vec::new(),
            asset_ids: Vec::new(),
            player_ids: Vec::new(),
            colors: Vec::new(),
            integrity: Vec::new(),
            ammunition: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            angular_velocities: Vec::new(),
            inertia: Vec::new(),
            masses: Vec::new(),
            boats: 0,
        };

        let masses: [f32; 2] = [100., 100.];

        let starting_positions = [Vector3::new(-2., 0., -15.0), Vector3::new(2., 0., -15.0)];

        let starting_velocities = [Vector3::<f32>::zeros(), Vector3::<f32>::zeros()];

        let starting_angular_velocities = [Vector3::<f32>::zeros(), Vector3::<f32>::zeros()];

        let moments_of_interia: Vec<Matrix3<f32>> = masses
            .iter()
            .map(|&m| Matrix3::from_diagonal(&Vector3::new(m / 6.0, m / 6.0, m / 6.0)))
            .collect();

        players
            .player_ids
            .into_iter()
            .zip(players.player_colors)
            .zip(masses)
            .zip(starting_positions)
            .zip(starting_velocities)
            .zip(starting_angular_velocities)
            .zip(moments_of_interia)
            .for_each(
                |(
                    (
                        ((((player_id, color), mass), position), initial_velocity),
                        initial_angular_velocity,
                    ),
                    moment_of_inertia,
                )| {
                    store.new_boat(
                        player_id,
                        color,
                        mass,
                        position,
                        initial_velocity,
                        initial_angular_velocity,
                        moment_of_inertia,
                    )
                },
            );

        store
    }

    pub fn new_boat(
        &mut self,
        player_id: PlayerId,
        color: Color,
        mass: f32,
        initial_position: Vector3<f32>,
        initial_velocity: Vector3<f32>,
        initial_angular_velocity: Vector3<f32>,
        moment_of_inertia: Matrix3<f32>,
    ) {
        let entity_id = EntityId::new();
        let boat_id = BoatId::new(entity_id);
        self.index.insert(boat_id, self.boats);
        self.entity_ids.push(entity_id);
        self.asset_ids.push(AssetId(self.boats));
        self.player_ids.push(player_id);
        self.colors.push(color);
        self.integrity.push_mut(DamageUnit(100));
        self.ammunition.push_mut(Ammo(1000));
        self.masses.push_mut(mass);
        self.positions.push_mut(initial_position);
        self.velocities.push_mut(initial_velocity);
        self.angular_velocities.push_mut(initial_angular_velocity);
        self.inertia.push_mut(moment_of_inertia);
        self.boats += 1;
    }
}

pub struct BulletStore {
    entity_ids: Vec<EntityId>,
    asset_ids: Vec<Asset>,
    owners: Vec<PlayerId>,
    damage: Vec<DamageUnit>,
}

impl BulletStore {
    pub fn new() -> Self {
        Self {
            entity_ids: Vec::new(),
            asset_ids: Vec::new(),
            owners: Vec::new(),
            damage: Vec::new(),
        }
    }
}

fn new_id_from_name(name: &'static str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, name.as_bytes())
}

fn new_id() -> Uuid {
    Uuid::now_v7()
}
