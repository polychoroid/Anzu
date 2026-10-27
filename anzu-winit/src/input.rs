use anzu_runtime::input::{ButtonPhase, InputAdapter, InputSignal, Key};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct WinitInputAdapter;
impl InputAdapter for WinitInputAdapter {
    type Command = KeyEvent;
    type CommandState = ElementState;

    fn map_input(&self, command: Self::Command) -> Option<InputSignal> {
        match command.physical_key {
            PhysicalKey::Code(key_code) => match key_code {
                KeyCode::KeyA => Some(InputSignal::Keyboard {
                    key: Key::Character('a'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyB => Some(InputSignal::Keyboard {
                    key: Key::Character('b'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyC => Some(InputSignal::Keyboard {
                    key: Key::Character('c'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyD => Some(InputSignal::Keyboard {
                    key: Key::Character('d'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyE => Some(InputSignal::Keyboard {
                    key: Key::Character('e'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyF => Some(InputSignal::Keyboard {
                    key: Key::Character('f'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyG => Some(InputSignal::Keyboard {
                    key: Key::Character('g'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyH => Some(InputSignal::Keyboard {
                    key: Key::Character('h'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyI => Some(InputSignal::Keyboard {
                    key: Key::Character('i'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyJ => Some(InputSignal::Keyboard {
                    key: Key::Character('j'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyK => Some(InputSignal::Keyboard {
                    key: Key::Character('k'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyL => Some(InputSignal::Keyboard {
                    key: Key::Character('l'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyM => Some(InputSignal::Keyboard {
                    key: Key::Character('m'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyN => Some(InputSignal::Keyboard {
                    key: Key::Character('n'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyO => Some(InputSignal::Keyboard {
                    key: Key::Character('o'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyP => Some(InputSignal::Keyboard {
                    key: Key::Character('p'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyQ => Some(InputSignal::Keyboard {
                    key: Key::Character('q'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyR => Some(InputSignal::Keyboard {
                    key: Key::Character('r'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyS => Some(InputSignal::Keyboard {
                    key: Key::Character('s'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyT => Some(InputSignal::Keyboard {
                    key: Key::Character('t'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyU => Some(InputSignal::Keyboard {
                    key: Key::Character('u'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyV => Some(InputSignal::Keyboard {
                    key: Key::Character('v'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyW => Some(InputSignal::Keyboard {
                    key: Key::Character('w'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyX => Some(InputSignal::Keyboard {
                    key: Key::Character('x'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyY => Some(InputSignal::Keyboard {
                    key: Key::Character('y'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::KeyZ => Some(InputSignal::Keyboard {
                    key: Key::Character('z'),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::Backspace => Some(InputSignal::Keyboard {
                    key: Key::Backspace,
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::Enter => Some(InputSignal::Keyboard {
                    key: Key::Enter,
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::Space => Some(InputSignal::Keyboard {
                    key: Key::Character(' '),
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::ArrowDown => Some(InputSignal::Keyboard {
                    key: Key::ArrowDown,
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::ArrowLeft => Some(InputSignal::Keyboard {
                    key: Key::ArrowLeft,
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::ArrowRight => Some(InputSignal::Keyboard {
                    key: Key::ArrowRight,
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::ArrowUp => Some(InputSignal::Keyboard {
                    key: Key::ArrowUp,
                    phase: self.map_input_state(command.state),
                }),
                KeyCode::Escape => Some(InputSignal::Keyboard {
                    key: Key::Escape,
                    phase: self.map_input_state(command.state),
                }),
                _ => None,
            },
            PhysicalKey::Unidentified(_native_key_code) => None,
        }
    }

    fn map_input_state(&self, command_state: Self::CommandState) -> ButtonPhase {
        match command_state {
            ElementState::Pressed => ButtonPhase::Pressed,
            ElementState::Released => ButtonPhase::Released,
        }
    }
}
