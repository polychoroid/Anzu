use crate::input::InputSignal;
use crate::server::Protocol;

pub trait Simulation<P: Protocol> {
    type Action;

    fn new() -> Self;
    fn apply(&mut self, command: Self::Action);
    fn step(&mut self);
    fn publish_updates(&mut self) -> Option<Vec<P::Update>>;
    fn adapt_signal(&self, signal: InputSignal) -> Option<Self::Action>;
    fn snapshot(&self) -> P::State;
}
