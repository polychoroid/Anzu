use std::f32::consts::{PI, TAU};
use web_time::Instant;

use anzu_runtime::assets::{AssetHandle, AssetId, DrawAssetInstance};
use anzu_runtime::input::InputSignal;
use anzu_runtime::server::{Protocol, State};
use anzu_runtime::simulation::Simulation;
use log::info;
use nalgebra::{Matrix4, Rotation3, Translation3, Vector3};

mod entities;

use crate::entities::{BoatStore, BulletStore, PlayerId, PlayerStore};

const SPIN_RADIANS_PER_SECOND: f32 = 0.75;

#[derive(Debug)]
pub enum Actions {
    North(PlayerId),
    East(PlayerId),
    South(PlayerId),
    West(PlayerId),
    Fire(PlayerId),
}

pub struct GameState {
    pub tick: u64,
    start_time: Instant,
    players: PlayerStore,
    boats: BoatStore,
    bullets: BulletStore,
    display_width: u32,
    display_height: u32,
}

impl State for GameState {
    type Update = GameStateUpdate;
    fn new() -> Self {
        let player_names = ["Player 0", "Player 1"];
        let players = PlayerStore::new([
            PlayerId::new(player_names[0]),
            PlayerId::new(player_names[1]),
        ]);

        let boats = BoatStore::new(&players);

        Self {
            tick: 0,
            start_time: Instant::now(),
            players: players,
            boats: boats,
            bullets: BulletStore::new(),
            display_width: 1,
            display_height: 1,
        }
    }
    fn apply_update(&mut self, _update: Self::Update) {
        todo!()
    }

    fn set_display_size(&mut self, width: u32, height: u32) {
        self.display_width = width;
        self.display_height = height;
    }

    fn project_display(&self) -> Vec<DrawAssetInstance> {
        let angle =
            (self.start_time.elapsed().as_secs_f32() * SPIN_RADIANS_PER_SECOND).rem_euclid(TAU);

        let motion = Translation3::new(0.0, 0.0, -5.0).to_homogeneous()
            * Rotation3::from_axis_angle(&Vector3::z_axis(), 23.5_f32.to_radians())
                .to_homogeneous()
            * Rotation3::from_axis_angle(&Vector3::x_axis(), 6.5_f32.to_radians()).to_homogeneous()
            * Rotation3::from_axis_angle(&Vector3::y_axis(), angle).to_homogeneous()
            * Matrix4::new_scaling(1.0);

        let aspect_ratio = self.display_width as f32 / self.display_height as f32;
        let projection = Matrix4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
        ) * nalgebra::Perspective3::new(aspect_ratio, PI / 3.0, 0.1, 100.0)
            .to_homogeneous();

        let transform = projection * motion;

        vec![
            DrawAssetInstance {
                asset_id: AssetId(0),
                transform: transform.into(),
            },
            DrawAssetInstance {
                asset_id: AssetId(1),
                transform: transform.into(),
            },
            DrawAssetInstance {
                asset_id: AssetId(2),
                transform: transform.into(),
            },
        ]
    }
}

impl Protocol for BoatTouching {
    type Update = GameStateUpdate;
    type State = GameState;
}

pub struct GameStateUpdate {
    player: PlayerId,
}

pub struct BoatTouching {
    gamestate: GameState,
    update_queue: Vec<GameStateUpdate>,
}

impl Simulation<BoatTouching> for BoatTouching {
    type Action = Actions;

    fn new() -> BoatTouching {
        let fresh_state = GameState::new();

        BoatTouching {
            gamestate: fresh_state,
            update_queue: Vec::new(),
        }
    }

    fn adapt_signal(&self, signal: anzu_runtime::input::InputSignal) -> Option<Self::Action> {
        match signal {
            InputSignal::Keyboard { key, phase } => match phase {
                anzu_runtime::input::ButtonPhase::Pressed => match key {
                    anzu_runtime::input::Key::Character(char) => match char {
                        'w' => Some(Actions::North(self.gamestate.players.player_ids[0])),
                        'a' => Some(Actions::East(self.gamestate.players.player_ids[0])),
                        's' => Some(Actions::South(self.gamestate.players.player_ids[0])),
                        'd' => Some(Actions::West(self.gamestate.players.player_ids[0])),
                        _ => None,
                    },
                    anzu_runtime::input::Key::ArrowUp => {
                        Some(Actions::North(self.gamestate.players.player_ids[1]))
                    }
                    anzu_runtime::input::Key::ArrowDown => {
                        Some(Actions::South(self.gamestate.players.player_ids[1]))
                    }
                    anzu_runtime::input::Key::ArrowLeft => {
                        Some(Actions::West(self.gamestate.players.player_ids[1]))
                    }
                    anzu_runtime::input::Key::ArrowRight => {
                        Some(Actions::East(self.gamestate.players.player_ids[1]))
                    }
                    anzu_runtime::input::Key::Backspace => None,
                    anzu_runtime::input::Key::Enter => {
                        Some(Actions::Fire(self.gamestate.players.player_ids[1]))
                    }
                    anzu_runtime::input::Key::Escape => None,
                    anzu_runtime::input::Key::Space => {
                        Some(Actions::Fire(self.gamestate.players.player_ids[0]))
                    }
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn apply(&mut self, action: Self::Action) {
        info!("{:?}", action);
    }

    fn load_display_assets(&self) -> Vec<AssetHandle> {
        info!("Sending asset paths to server for display");

        vec![
            AssetHandle {
                id: AssetId(0),
                bytes: include_bytes!("assets/coast_guard_patrol_boat_3d_model.glb"),
            },
            AssetHandle {
                id: AssetId(1),
                bytes: include_bytes!("assets/anzu_textured_cube.glb"),
            },
            AssetHandle {
                id: AssetId(2),
                bytes: include_bytes!("assets/red_cube.glb"),
            },
        ]
    }

    fn publish_updates(&mut self) -> Option<Vec<GameStateUpdate>> {
        match self.update_queue.is_empty() {
            false => Some(std::mem::take(&mut self.update_queue)),
            true => None,
        }
    }

    fn step(&mut self) {
        self.gamestate.tick += 1;
    }
}
