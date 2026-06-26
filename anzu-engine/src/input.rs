#[derive(Clone, Debug, PartialEq)]
pub struct InputEvent {
    pub source: String,
    pub control: String,
    pub value: f32,
    pub device_index: Option<u32>,
    pub is_pressed: bool,
    pub is_repeat: bool,
}
