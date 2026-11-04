use crate::assets::AssetHandle;
use crate::input::InputSignal;
use crate::server::Protocol;

pub trait Simulation<P: Protocol> {
    type Action;

    fn new() -> Self;
    fn adapt_signal(&self, signal: InputSignal) -> Option<Self::Action>;
    fn apply(&mut self, command: Self::Action);
    fn load_display_assets(&self) -> Vec<AssetHandle>;
    fn publish_updates(&mut self) -> Option<Vec<P::Update>>;
    fn step(&mut self);
}
