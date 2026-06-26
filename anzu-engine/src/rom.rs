use crate::ecs::{EntityId, World};
use crate::simulation::SimulationModel;

pub trait RomPackage {
    fn rom_id(&self) -> &'static str;
    fn bootstrap_world(&self, world: &mut World) -> EntityId;
    fn create_simulation(&self, world: &World, anchor_entity: EntityId) -> Box<dyn SimulationModel>;
}
