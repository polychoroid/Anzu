use crate::{display::DrawBatch, input::InputSignal, simulation::Simulation};
use log::debug;
use std::collections::VecDeque;
use web_time::Instant;

pub trait State {
    type Update;

    fn new() -> Self;
    fn apply_update(&mut self, update: Self::Update);
    fn project_display(&self) -> Vec<DrawBatch>;
}

pub trait Protocol {
    type Update;
    type State: State<Update = Self::Update>;
}

pub trait Server {
    type Protocol: Protocol;

    fn tick(&mut self, tick: Instant);
    fn queue_input(&mut self, input: InputSignal);
    fn dequeue_update_batch(&mut self) -> Option<Vec<<Self::Protocol as Protocol>::Update>>;
}

pub struct LocalServer<P: Protocol, S: Simulation<P>> {
    simulation: S,
    input_queue: Vec<InputSignal>,
    update_queue: VecDeque<Vec<P::Update>>,
    last_update: Instant,
}

impl<P: Protocol, S: Simulation<P>> Server for LocalServer<P, S> {
    type Protocol = P;

    fn tick(&mut self, tick: Instant) {
        self.update(tick)
    }

    fn queue_input(&mut self, input: InputSignal) {
        debug!("Input queued: {:?}", &input);
        self.input_queue.push(input)
    }

    fn dequeue_update_batch(&mut self) -> Option<Vec<P::Update>> {
        self.update_queue.pop_front()
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
