pub trait InputAdapter {
    type Command;

    fn map_input(&self, command: Self::Command) -> Option<InputSignal>;
    fn map_input_state(&self, command: Self::Command) -> ButtonPhase;
}

#[derive(Debug)]
pub enum Key {
    Character(char),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Backspace,
    Enter,
    Escape,
    Space,
}

#[derive(Debug)]
pub enum ButtonPhase {
    Pressed,
    Released,
    Repeated,
}

#[derive(Debug)]
pub enum GamepadEvent {
    Button {
        button: GamepadDPad,
        phase: ButtonPhase,
    },
    Axis {
        axis: GamepadAxis,
        value: f32,
    },
}

#[derive(Debug)]
pub enum GamepadDPad {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub enum GamepadAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
}

#[derive(Debug)]
pub enum InputSignal {
    Keyboard { key: Key, phase: ButtonPhase },
    Text(String),
    GamePad { device_id: u32, event: GamepadEvent },
}
