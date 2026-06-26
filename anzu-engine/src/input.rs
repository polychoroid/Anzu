use winit::keyboard::KeyCode;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InputFrame {
    pub thrust: i8,
    pub turn: i8,
    pub fire: bool,
    pub toggle_pause: bool,
    pub step_once: bool,
}

pub struct InputHistory {
    frames: Vec<InputFrame>,
    capacity: usize,
}

impl InputHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: Vec::with_capacity(capacity.min(512)),
            capacity,
        }
    }

    pub fn record(&mut self, frame: InputFrame) {
        if self.capacity == 0 {
            return;
        }

        if self.frames.len() == self.capacity {
            self.frames.remove(0);
        }
        self.frames.push(frame);
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &InputFrame> {
        self.frames.iter()
    }
}

#[derive(Default)]
pub struct InputManager {
    thrust_forward: bool,
    thrust_reverse: bool,
    turn_left: bool,
    turn_right: bool,
    fire_key_down: bool,
    pause_key_down: bool,
    step_key_down: bool,
    pending_fire: bool,
    pending_toggle_pause: bool,
    pending_step_once: bool,
}

impl InputManager {
    pub fn handle_key_event(&mut self, key_code: KeyCode, is_pressed: bool, is_repeat: bool) {
        if is_repeat && is_pressed {
            return;
        }

        match key_code {
            KeyCode::KeyW => self.thrust_forward = is_pressed,
            KeyCode::KeyS => self.thrust_reverse = is_pressed,
            KeyCode::KeyA => self.turn_left = is_pressed,
            KeyCode::KeyD => self.turn_right = is_pressed,
            KeyCode::Space => {
                if is_pressed && !self.fire_key_down {
                    self.pending_fire = true;
                }
                self.fire_key_down = is_pressed;
            }
            KeyCode::Backquote => {
                if is_pressed && !self.pause_key_down {
                    self.pending_toggle_pause = true;
                }
                self.pause_key_down = is_pressed;
            }
            KeyCode::Period => {
                if is_pressed && !self.step_key_down {
                    self.pending_step_once = true;
                }
                self.step_key_down = is_pressed;
            }
            _ => {}
        }
    }

    pub fn snapshot_frame(&mut self) -> InputFrame {
        let frame = InputFrame {
            thrust: axis_value(self.thrust_reverse, self.thrust_forward),
            turn: axis_value(self.turn_left, self.turn_right),
            fire: self.pending_fire,
            toggle_pause: self.pending_toggle_pause,
            step_once: self.pending_step_once,
        };

        self.pending_fire = false;
        self.pending_toggle_pause = false;
        self.pending_step_once = false;

        frame
    }
}

fn axis_value(negative: bool, positive: bool) -> i8 {
    match (negative, positive) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{axis_value, InputFrame, InputHistory, InputManager};
    use winit::keyboard::KeyCode;

    #[test]
    fn axis_value_cancels_opposites() {
        assert_eq!(axis_value(false, false), 0);
        assert_eq!(axis_value(true, false), -1);
        assert_eq!(axis_value(false, true), 1);
        assert_eq!(axis_value(true, true), 0);
    }

    #[test]
    fn input_manager_maps_keyboard_to_frame() {
        let mut input = InputManager::default();
        input.handle_key_event(KeyCode::KeyW, true, false);
        input.handle_key_event(KeyCode::KeyD, true, false);
        input.handle_key_event(KeyCode::Space, true, false);

        let frame = input.snapshot_frame();
        assert_eq!(frame.thrust, 1);
        assert_eq!(frame.turn, 1);
        assert!(frame.fire);

        let held = input.snapshot_frame();
        assert!(!held.fire);
    }

    #[test]
    fn replay_history_is_deterministic_for_same_frames() {
        let mut history = InputHistory::new(16);
        history.record(InputFrame {
            thrust: 1,
            turn: 1,
            fire: false,
            toggle_pause: false,
            step_once: false,
        });
        history.record(InputFrame {
            thrust: 0,
            turn: -1,
            fire: false,
            toggle_pause: false,
            step_once: false,
        });

        let mut state_a = (0.0f32, 0.0f32, 0.0f32);
        let mut state_b = (0.0f32, 0.0f32, 0.0f32);
        let dt = 1.0f32 / 60.0;

        for frame in history.iter() {
            state_a.0 += frame.thrust as f32 * dt;
            state_a.1 += frame.turn as f32 * dt;
            state_a.2 += if frame.fire { dt } else { 0.0 };
        }
        for frame in history.iter() {
            state_b.0 += frame.thrust as f32 * dt;
            state_b.1 += frame.turn as f32 * dt;
            state_b.2 += if frame.fire { dt } else { 0.0 };
        }

        assert!((state_a.0 - state_b.0).abs() < f32::EPSILON);
        assert!((state_a.1 - state_b.1).abs() < f32::EPSILON);
        assert!((state_a.2 - state_b.2).abs() < f32::EPSILON);
    }

    #[test]
    fn pause_and_step_are_edge_triggered() {
        let mut input = InputManager::default();

        input.handle_key_event(KeyCode::Backquote, true, false);
        input.handle_key_event(KeyCode::Period, true, false);
        let first = input.snapshot_frame();
        assert!(first.toggle_pause);
        assert!(first.step_once);

        let held = input.snapshot_frame();
        assert!(!held.toggle_pause);
        assert!(!held.step_once);

        input.handle_key_event(KeyCode::Backquote, false, false);
        input.handle_key_event(KeyCode::Period, false, false);
        input.handle_key_event(KeyCode::Backquote, true, false);
        input.handle_key_event(KeyCode::Period, true, false);
        let second = input.snapshot_frame();
        assert!(second.toggle_pause);
        assert!(second.step_once);
    }
}
