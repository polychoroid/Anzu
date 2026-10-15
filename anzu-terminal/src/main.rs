use anzu_rom_you_touched_my_boats::BoatTouching;
use anzu_runtime::server::{LocalServer, Server};
use anzu_runtime::session::Session;
use anzu_runtime::simulation::Simulation;
use crossterm::event::{self, Event};
use env_logger;
use std::io::Write;
use std::time::Duration;

mod input;
use crate::input::CrossTermInput;

fn main() -> Result<(), ()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}\r",
                buf.timestamp(),
                record.level(),
                record.args()
            )
        })
        .init();
    crossterm::terminal::enable_raw_mode().unwrap();
    let simulation = BoatTouching::new();
    let server = LocalServer::<BoatTouching>::new(simulation);
    let mut session =
        Session::<CrossTermInput, BoatTouching>::new(CrossTermInput, Server::Local(server));

    let mut run = true;

    while run {
        session.tick();
        if event::poll(Duration::from_millis(1000)).unwrap() {
            let Event::Key(key_event) = event::read().unwrap() else {
                continue;
            };
            if key_event.code.is_esc() {
                run = false;
            }
            session.adapt_input(key_event);
        }
    }

    crossterm::terminal::disable_raw_mode().unwrap();
    println!("Goodbye");
    Ok(())
}
