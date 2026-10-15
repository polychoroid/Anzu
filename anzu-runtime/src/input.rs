pub trait InputAdapter {
    type Command;
    type CommandState;

    fn map_input(&self, command: Self::Command) -> Option<InputSignal>;
    fn map_input_state(&self, command_state: Self::CommandState) -> ButtonPhase;
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
}

#[derive(Debug)]
pub enum ButtonPhase {
    Pressed,
    Released,
    Repeated,
}

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

pub enum GamepadDPad {
    Up,
    Down,
    Left,
    Right,
}

pub enum GamepadAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
}

pub enum InputSignal {
    Keyboard { key: Key, phase: ButtonPhase },
    Text(String),
    GamePad { device_id: u32, event: GamepadEvent },
}
