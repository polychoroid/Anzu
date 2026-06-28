#[derive(Clone, Debug, PartialEq)]
pub struct InputEvent {
    pub source: String,
    pub control: String,
    pub value: f32,
    pub device_index: Option<u32>,
    pub is_pressed: bool,
    pub is_repeat: bool,
}

#[cfg(test)]
mod tests {
    use super::InputEvent;

    #[test]
    fn constructs_input_event() {
        let event = InputEvent {
            source: "gamepad".to_owned(),
            control: "right_trigger".to_owned(),
            value: 0.75,
            device_index: Some(0),
            is_pressed: true,
            is_repeat: false,
        };

        assert_eq!(event.source, "gamepad");
        assert_eq!(event.control, "right_trigger");
        assert_eq!(event.value, 0.75);
        assert_eq!(event.device_index, Some(0));
        assert!(event.is_pressed);
        assert!(!event.is_repeat);
    }
}
