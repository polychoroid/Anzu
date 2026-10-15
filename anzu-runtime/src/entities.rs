use uuid::Uuid;

#[derive(PartialEq, Debug, Clone)]
pub enum ActorModes {
    ActiveCpu,
    ActiveUser,
}

#[derive(Clone)]
pub struct Actor {
    pub name: String,
    pub id: Uuid,
    pub mode: ActorModes,
}

impl Actor {
    pub fn new(name: String, mode: ActorModes) -> Actor {
        return Actor {
            id: Uuid::new_v5(&Uuid::NAMESPACE_URL, name.as_bytes()),
            name: name,
            mode: mode,
        };
    }
}
