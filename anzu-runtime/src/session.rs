use crate::input::InputAdapter;
use crate::server::Server;
use crate::simulation::Simulation;
use std::time::Instant;

pub struct Session<Input: InputAdapter, Sim: Simulation> {
    input_adapter: Input,
    server: Server<Sim>,
}

impl<Input: InputAdapter, Sim: Simulation> Session<Input, Sim> {
    pub fn new(input: Input, server: Server<Sim>) -> Session<Input, Sim> {
        Session {
            input_adapter: input,
            server: server,
        }
    }

    pub fn adapt_input(&mut self, input: Input::Command) {
        self.server
            .queue_input(self.input_adapter.map_input(input).unwrap());
    }

    pub fn tick(&mut self) {
        self.server.send_session_tick(Instant::now());
    }
}
