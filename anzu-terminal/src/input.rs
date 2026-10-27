use anzu_runtime::input::{ButtonPhase, InputAdapter, InputSignal, Key};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

pub struct CrossTermInput;

impl InputAdapter for CrossTermInput {
    type Command = KeyEvent;

    fn map_input(&self, command: Self::Command) -> Option<InputSignal> {
        match &command.code {
            KeyCode::Char(c) => match &c {
                ' ' => Some(InputSignal::Keyboard {
                    key: Key::Space,
                    phase: self.map_input_state(command),
                }),
                _ => Some(InputSignal::Keyboard {
                    key: Key::Character(*c),
                    phase: self.map_input_state(command),
                }),
            },
            KeyCode::Backspace => Some(InputSignal::Keyboard {
                key: Key::Backspace,
                phase: self.map_input_state(command),
            }),
            KeyCode::Enter => Some(InputSignal::Keyboard {
                key: Key::Enter,
                phase: self.map_input_state(command),
            }),
            KeyCode::Left => Some(InputSignal::Keyboard {
                key: Key::ArrowLeft,
                phase: self.map_input_state(command),
            }),
            KeyCode::Right => Some(InputSignal::Keyboard {
                key: Key::ArrowRight,
                phase: self.map_input_state(command),
            }),
            KeyCode::Up => Some(InputSignal::Keyboard {
                key: Key::ArrowUp,
                phase: self.map_input_state(command),
            }),
            KeyCode::Down => Some(InputSignal::Keyboard {
                key: Key::ArrowDown,
                phase: self.map_input_state(command),
            }),
            KeyCode::Home => None,
            KeyCode::End => None,
            KeyCode::PageUp => None,
            KeyCode::PageDown => None,
            KeyCode::Tab => None,
            KeyCode::BackTab => None,
            KeyCode::Delete => None,
            KeyCode::Insert => None,
            KeyCode::F(_) => None,
            KeyCode::Null => None,
            KeyCode::Esc => Some(InputSignal::Keyboard {
                key: Key::Escape,
                phase: self.map_input_state(command),
            }),
            KeyCode::CapsLock => None,
            KeyCode::ScrollLock => None,
            KeyCode::NumLock => None,
            KeyCode::PrintScreen => None,
            KeyCode::Pause => None,
            KeyCode::Menu => None,
            KeyCode::KeypadBegin => None,
            KeyCode::Media(_media_key_code) => None,
            KeyCode::Modifier(_modifier_key_code) => None,
        }
    }

    fn map_input_state(&self, command: Self::Command) -> ButtonPhase {
        match command.kind {
            KeyEventKind::Press => ButtonPhase::Pressed,
            KeyEventKind::Repeat => ButtonPhase::Repeated,
            KeyEventKind::Release => ButtonPhase::Released,
        }
    }
}
