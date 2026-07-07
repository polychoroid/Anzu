use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct InputEvent {
    pub source: String,
    pub control: String,
    pub value: f32,
    pub device_index: Option<u32>,
    pub is_pressed: bool,
    pub is_repeat: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InputBinding {
    pub source: &'static str,
    pub control: &'static str,
}

impl InputBinding {
    pub const fn new(source: &'static str, control: &'static str) -> Self {
        Self { source, control }
    }

    pub const fn keyboard(control: &'static str) -> Self {
        Self::new("keyboard", control)
    }

    pub const fn mouse(control: &'static str) -> Self {
        Self::new("mouse", control)
    }

    pub const fn gamepad(control: &'static str) -> Self {
        Self::new("gamepad", control)
    }
}

pub const RESERVED_OVERLAY_TOGGLE_BINDING: InputBinding = InputBinding::keyboard("Escape");

pub fn is_reserved_rom_binding(binding: InputBinding) -> bool {
    binding == RESERVED_OVERLAY_TOGGLE_BINDING
}

pub struct InputControlMap<Action> {
    bindings: Vec<(InputBinding, Action)>,
}

impl<Action> Default for InputControlMap<Action> {
    fn default() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }
}

impl<Action> InputControlMap<Action> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, binding: InputBinding, action: Action) {
        if let Some((_, existing_action)) =
            self.bindings.iter_mut().find(|(existing_binding, _)| {
                existing_binding.source == binding.source
                    && existing_binding.control == binding.control
            })
        {
            *existing_action = action;
            return;
        }

        self.bindings.push((binding, action));
    }

    pub fn bind_rom_action(
        &mut self,
        binding: InputBinding,
        action: Action,
    ) -> Result<(), &'static str> {
        if is_reserved_rom_binding(binding) {
            return Err("Escape is reserved for the engine overlay context");
        }

        self.bind(binding, action);
        Ok(())
    }

    pub fn resolve(&self, event: &InputEvent) -> Option<&Action> {
        self.bindings
            .iter()
            .find(|(binding, _)| {
                binding.source == event.source.as_str() && binding.control == event.control.as_str()
            })
            .map(|(_, action)| action)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputActionEvent<Action> {
    pub context_id: &'static str,
    pub action: Action,
    pub value: f32,
    pub device_index: Option<u32>,
    pub is_pressed: bool,
    pub is_repeat: bool,
}

pub struct InputContextStack<Action> {
    contexts: BTreeMap<&'static str, InputControlMap<Action>>,
    stack: Vec<&'static str>,
}

impl<Action> InputContextStack<Action> {
    pub fn new(root_context_id: &'static str, root_map: InputControlMap<Action>) -> Self {
        let mut contexts = BTreeMap::new();
        contexts.insert(root_context_id, root_map);
        Self {
            contexts,
            stack: vec![root_context_id],
        }
    }

    pub fn register_context(&mut self, context_id: &'static str, map: InputControlMap<Action>) {
        self.contexts.insert(context_id, map);
    }

    pub fn active_context(&self) -> &'static str {
        self.stack.last().copied().unwrap_or_default()
    }

    pub fn push_context(&mut self, context_id: &'static str) -> bool {
        if !self.contexts.contains_key(context_id) {
            return false;
        }

        self.stack.push(context_id);
        true
    }

    pub fn pop_context(&mut self) -> bool {
        if self.stack.len() <= 1 {
            return false;
        }

        self.stack.pop();
        true
    }

    pub fn replace_active_context(&mut self, context_id: &'static str) -> bool {
        if !self.contexts.contains_key(context_id) {
            return false;
        }

        if let Some(active) = self.stack.last_mut() {
            *active = context_id;
            true
        } else {
            false
        }
    }
}

impl<Action: Clone> InputContextStack<Action> {
    pub fn resolve_event(&self, event: &InputEvent) -> Option<InputActionEvent<Action>> {
        let context_id = self.active_context();
        let action = self.contexts.get(context_id)?.resolve(event)?.clone();

        Some(InputActionEvent {
            context_id,
            action,
            value: event.value,
            device_index: event.device_index,
            is_pressed: event.is_pressed,
            is_repeat: event.is_repeat,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InputBinding, InputContextStack, InputControlMap, InputEvent, is_reserved_rom_binding,
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum TestAction {
        Jump,
        Confirm,
    }

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

    #[test]
    fn reserved_overlay_binding_is_rejected_for_rom_maps() {
        let mut map = InputControlMap::new();

        assert!(is_reserved_rom_binding(InputBinding::keyboard("Escape")));
        assert!(
            map.bind_rom_action(InputBinding::keyboard("Escape"), TestAction::Confirm)
                .is_err()
        );
    }

    #[test]
    fn active_context_controls_which_action_is_resolved() {
        let mut gameplay = InputControlMap::new();
        gameplay
            .bind_rom_action(InputBinding::keyboard("Space"), TestAction::Jump)
            .expect("Space should be legal for gameplay");

        let mut menu = InputControlMap::new();
        menu.bind_rom_action(InputBinding::keyboard("Space"), TestAction::Confirm)
            .expect("Space should be legal for menu");

        let mut stack = InputContextStack::new("gameplay", gameplay);
        stack.register_context("menu", menu);

        let space_pressed = InputEvent {
            source: "keyboard".to_owned(),
            control: "Space".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        };

        assert_eq!(stack.active_context(), "gameplay");
        assert_eq!(
            stack
                .resolve_event(&space_pressed)
                .map(|event| event.action),
            Some(TestAction::Jump)
        );

        assert!(stack.push_context("menu"));
        assert_eq!(stack.active_context(), "menu");
        assert_eq!(
            stack
                .resolve_event(&space_pressed)
                .map(|event| event.action),
            Some(TestAction::Confirm)
        );

        assert!(stack.pop_context());
        assert_eq!(stack.active_context(), "gameplay");
        assert_eq!(
            stack
                .resolve_event(&space_pressed)
                .map(|event| event.action),
            Some(TestAction::Jump)
        );
    }
}
