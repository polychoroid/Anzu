use anzu_runtime::input::{ButtonPhase, InputAdapter, InputSignal, Key};
use log::info;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::NamedKey,
};

pub struct WinitInputAdapter;
impl InputAdapter for WinitInputAdapter {
    type Command = KeyEvent;

    fn map_input(&self, command: Self::Command) -> Option<InputSignal> {
        info!("Mapping input: {:?}", &command);
        match &command.logical_key {
            winit::keyboard::Key::Named(named_key) => match named_key {
                NamedKey::Enter => Some(InputSignal::Keyboard {
                    key: Key::Enter,
                    phase: self.map_input_state(command),
                }),
                NamedKey::Tab => todo!(),
                NamedKey::Space => Some(InputSignal::Keyboard {
                    key: Key::Space,
                    phase: self.map_input_state(command),
                }),
                NamedKey::ArrowDown => Some(InputSignal::Keyboard {
                    key: Key::ArrowDown,
                    phase: self.map_input_state(command),
                }),
                NamedKey::ArrowLeft => Some(InputSignal::Keyboard {
                    key: Key::ArrowLeft,
                    phase: self.map_input_state(command),
                }),
                NamedKey::ArrowRight => Some(InputSignal::Keyboard {
                    key: Key::ArrowRight,
                    phase: self.map_input_state(command),
                }),
                NamedKey::ArrowUp => Some(InputSignal::Keyboard {
                    key: Key::ArrowUp,
                    phase: self.map_input_state(command),
                }),
                NamedKey::Backspace => Some(InputSignal::Keyboard {
                    key: Key::Backspace,
                    phase: self.map_input_state(command),
                }),
                NamedKey::Escape => Some(InputSignal::Keyboard {
                    key: Key::Escape,
                    phase: self.map_input_state(command),
                }),
                _ => None,
            },
            winit::keyboard::Key::Character(character) => Some(InputSignal::Keyboard {
                key: Key::Character(character.chars().next().unwrap()),
                phase: self.map_input_state(command),
            }),
            _ => None,
        }
    }

    fn map_input_state(&self, command: Self::Command) -> ButtonPhase {
        match command.repeat {
            true => ButtonPhase::Repeated,
            false => match command.state {
                ElementState::Pressed => ButtonPhase::Pressed,
                ElementState::Released => ButtonPhase::Released,
            },
        }
    }
}
