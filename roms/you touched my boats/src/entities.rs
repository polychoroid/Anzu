use nalgebra::Vector3;
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
    index: HashMap<BoatId, usize>,
    entity_ids: Vec<EntityId>,
    assets: Vec<Asset>,
    player_ids: Vec<PlayerId>,
    colors: Vec<Color>,
    integrity: Vec<DamageUnit>,
    ammunition: Vec<Ammo>,
    positions: Vec<Vector3<f32>>,
    velocities: Vec<Vector3<f32>>,
    boats: usize,
}

impl BoatStore {
    pub fn new(players: &PlayerStore) -> Self {
        let mut store = Self {
            index: HashMap::new(),
            entity_ids: Vec::new(),
            assets: Vec::new(),
            player_ids: Vec::new(),
            colors: Vec::new(),
            integrity: Vec::new(),
            ammunition: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            boats: 0,
        };

        let starting_positions: [Vector3<f32>; 2] =
            [Vector3::new(-1., 0., 0.), Vector3::new(1., 0., 0.)];

        players
            .player_ids
            .into_iter()
            .zip(players.player_colors)
            .zip(starting_positions)
            .for_each(|x| store.new_boat(x.0.0, x.0.1, x.1));

        store
    }

    pub fn new_boat(&mut self, player_id: PlayerId, color: Color, initial_position: Vector3<f32>) {
        let entity_id = EntityId::new();
        let boat_id = BoatId::new(entity_id);
        self.index.insert(boat_id, self.boats);
        self.entity_ids.push(entity_id);
        self.assets.push(Asset::Boat);
        self.player_ids.push(player_id);
        self.colors.push(color);
        self.integrity.push_mut(DamageUnit(100));
        self.ammunition.push_mut(Ammo(1000));
        self.positions.push_mut(initial_position);
        self.velocities.push_mut(Vector3::zeros());
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
