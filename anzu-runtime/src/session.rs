use crate::display::DisplayAdapter;
use crate::input::InputAdapter;
use crate::server::{Protocol, Server, State};
use log::info;
use web_time::Instant;

pub trait Session {
    type SessionInptuAdapter: InputAdapter;
    type SessionDisplayAdapter: DisplayAdapter;
    type SessionServer: Server;
    type SessionState: State;

    fn new(
        input_adapter: Self::SessionInptuAdapter,
        display_adapter: Self::SessionDisplayAdapter,
        server: Self::SessionServer,
    ) -> Self;
    fn attach_display_target(
        &mut self,
        target: <Self::SessionDisplayAdapter as DisplayAdapter>::Target,
    );
    fn load_display_data(&mut self) -> Result<(), std::io::Error>;
    fn queue_input(&mut self, input: <Self::SessionInptuAdapter as InputAdapter>::Command);
    fn resize_display(&mut self, width: u32, height: u32);
    fn request_redraw(&mut self);
    fn tick(&mut self);
}

pub struct SessionData<I: InputAdapter, D: DisplayAdapter, S: Server> {
    input_adapter: I,
    display_adapter: D,
    server: S,
    service_state: <S::Protocol as Protocol>::State,
}

impl<I: InputAdapter, D: DisplayAdapter, S: Server> Session for SessionData<I, D, S> {
    type SessionInptuAdapter = I;
    type SessionDisplayAdapter = D;
    type SessionServer = S;
    type SessionState = <S::Protocol as Protocol>::State;

    fn new(input_adapter: I, display_adapter: D, server: S) -> Self {
        Self {
            input_adapter: input_adapter,
            display_adapter: display_adapter,
            service_state: <S::Protocol as Protocol>::State::new(),
            server: server,
        }
    }

    fn attach_display_target(
        &mut self,
        target: <Self::SessionDisplayAdapter as DisplayAdapter>::Target,
    ) {
        self.display_adapter.attach_target(target);
    }

    fn load_display_data(&mut self) -> Result<(), std::io::Error> {
        self.display_adapter
            .load_assets(self.server.load_display_assets())?;
        info!("Request load assets from server");
        Ok(())
    }

    fn queue_input(&mut self, command: I::Command) {
        if let Some(input) = self.input_adapter.map_input(command) {
            self.server.queue_input(input);
        }
    }

    fn resize_display(&mut self, width: u32, height: u32) {
        self.display_adapter.resize(width, height);
        self.service_state.set_display_size(width, height);
    }

    fn request_redraw(&mut self) {
        self.display_adapter
            .render(self.service_state.project_display())
            .unwrap();
    }

    fn tick(&mut self) {
        self.server.tick(Instant::now());
    }
}
