use crate::{input::InputSignal, simulation::Simulation};
use log::info;
use std::time::Instant;

pub enum Server<S: Simulation> {
    Local(LocalServer<S>),
    Remote(RemoteServer),
}

impl<S: Simulation> Server<S> {
    pub fn queue_input(&mut self, input: InputSignal) {
        match self {
            Self::Local(local_server) => local_server.queue_command(input),
            _ => (),
        }
    }

    pub fn send_session_tick(&mut self, tick: Instant) {
        match self {
            Self::Local(local_server) => local_server.update(tick),
            _ => (),
        }
    }
}

pub struct LocalServer<S: Simulation> {
    pub snapshot: S::Snapshot,
    input_queue: Vec<S::Command>,
    simulation: S,
    last_update: Instant,
}

impl<S: Simulation> LocalServer<S> {
    pub fn new(simulation: S) -> LocalServer<S> {
        LocalServer {
            snapshot: simulation.snapshot(),
            input_queue: Vec::new(),
            simulation: simulation,
            last_update: Instant::now(),
        }
    }

    fn update(&mut self, tick: Instant) {
        info!("{}", self.input_queue.len());
        for command in self.input_queue.drain(0..) {
            self.simulation.apply(command);
        }
        self.simulation.step();
        self.snapshot = self.simulation.snapshot();
        self.last_update = tick;
    }

    pub fn queue_command(&mut self, signal: InputSignal) {
        if let Some(command) = self.simulation.signal_command(signal) {
            self.input_queue.push(command);
        }
    }

    pub fn queue_commands(&mut self) {
        todo!()
    }
}

pub struct RemoteServer {}
