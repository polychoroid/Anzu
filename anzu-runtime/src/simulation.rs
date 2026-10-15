use crate::input::InputSignal;

pub trait Simulation {
    type Command;
    type Snapshot;
    type WorldState;

    fn new() -> Self;
    fn apply(&mut self, command: Self::Command);
    fn step(&mut self);
    fn snapshot(&self) -> Self::Snapshot;
    fn signal_command(&self, signal: InputSignal) -> Option<Self::Command>;
}
