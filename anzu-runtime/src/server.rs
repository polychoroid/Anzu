use crate::{
    assets::{AssetHandle, DrawAssetInstance},
    input::InputSignal,
    simulation::Simulation,
};
use log::{debug, info};
use std::collections::VecDeque;
use web_time::Instant;

pub trait State {
    type Update;

    fn new() -> Self;
    fn apply_update(&mut self, update: Self::Update);
    fn project_display(&self) -> Vec<DrawAssetInstance>;
}

pub trait Protocol {
    type Update;
    type State: State<Update = Self::Update>;
}

pub trait Server {
    type Protocol: Protocol;

    fn dequeue_update_batch(&mut self) -> Option<Vec<<Self::Protocol as Protocol>::Update>>;
    fn load_display_assets(&self) -> Vec<AssetHandle>;
    fn queue_input(&mut self, input: InputSignal);
    fn tick(&mut self, tick: Instant);
}

pub struct LocalServer<P: Protocol, S: Simulation<P>> {
    simulation: S,
    input_queue: Vec<InputSignal>,
    update_queue: VecDeque<Vec<P::Update>>,
    last_update: Instant,
}

impl<P: Protocol, S: Simulation<P>> Server for LocalServer<P, S> {
    type Protocol = P;

    fn dequeue_update_batch(&mut self) -> Option<Vec<P::Update>> {
        self.update_queue.pop_front()
    }

    fn load_display_assets(&self) -> Vec<AssetHandle> {
        info!("Loading requested assets from simulation to session");
        self.simulation.load_display_assets()
    }

    fn queue_input(&mut self, input: InputSignal) {
        debug!("Input queued: {:?}", &input);
        self.input_queue.push(input)
    }

    fn tick(&mut self, tick: Instant) {
        self.update(tick)
    }
}

impl<P: Protocol, S: Simulation<P>> LocalServer<P, S> {
    pub fn new(simulation: S) -> Self {
        Self {
            simulation: simulation,
            input_queue: Vec::new(),
            update_queue: VecDeque::<Vec<P::Update>>::new(),
            last_update: Instant::now(),
        }
    }

    fn update(&mut self, tick: Instant) {
        debug!("{}", self.input_queue.len());

        for input in self.input_queue.drain(0..) {
            if let Some(command) = self.simulation.adapt_signal(input) {
                self.simulation.apply(command);
            }
        }

        self.simulation.step();
        self.last_update = tick;

        if let Some(update) = self.simulation.publish_updates() {
            self.update_queue.push_back(update)
        }
    }
}

pub struct RemoteServer;
pub struct PredictiveRemoteServer;
