use std::collections::{BTreeMap, VecDeque};

use winit::keyboard::KeyCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputContext {
    GameplayContext,
    OverlayContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlCommand {
    TogglePause,
    ToggleOverlay,
    ToggleControl,
    TogglePerformance,
    TogglePinnedFpsHud,
    ToggleNotifications,
    ResetGame,
    StepOneTick,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HudTelemetryMode {
    Hidden,
    Fps,
}

impl HudTelemetryMode {
    fn next(self) -> Self {
        match self {
            Self::Hidden => Self::Fps,
            Self::Fps => Self::Hidden,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Hidden => "hidden",
            Self::Fps => "fps",
        }
    }

    pub fn includes_fps(self) -> bool {
        matches!(self, Self::Fps)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OverlayVisibility {
    pub pause_menu: bool,
    pub control: bool,
    pub performance: bool,
    pub notifications: bool,
}

impl OverlayVisibility {
    pub fn new() -> Self {
        Self {
            pause_menu: false,
            control: false,
            performance: false,
            notifications: false,
        }
    }

    pub fn set_pause_menu(&mut self, visible: bool) {
        self.pause_menu = visible;
    }

    pub fn any_visible(&self) -> bool {
        self.pause_menu || self.control || self.performance || self.notifications
    }

    fn non_pause_overlay_visible(&self) -> bool {
        self.control || self.performance || self.notifications
    }
}

pub fn default_control_bindings() -> BTreeMap<KeyCode, ControlCommand> {
    let mut bindings = BTreeMap::new();
    bindings.insert(KeyCode::KeyP, ControlCommand::TogglePause);
    bindings.insert(KeyCode::Escape, ControlCommand::ToggleOverlay);
    bindings.insert(KeyCode::ArrowLeft, ControlCommand::ToggleControl);
    bindings.insert(KeyCode::ArrowRight, ControlCommand::TogglePerformance);
    bindings.insert(KeyCode::F3, ControlCommand::ToggleNotifications);
    bindings.insert(KeyCode::KeyF, ControlCommand::TogglePinnedFpsHud);
    bindings.insert(KeyCode::KeyR, ControlCommand::ResetGame);
    bindings
}

pub fn command_label(command: ControlCommand) -> &'static str {
    match command {
        ControlCommand::TogglePause => "pause",
        ControlCommand::ToggleOverlay => "overlay",
        ControlCommand::ToggleControl => "control",
        ControlCommand::TogglePerformance => "performance",
        ControlCommand::TogglePinnedFpsHud => "fps_hud",
        ControlCommand::ToggleNotifications => "notifications",
        ControlCommand::ResetGame => "reset",
        ControlCommand::StepOneTick => "step",
    }
}

pub fn parse_control_command(value: &str) -> Option<ControlCommand> {
    match value.trim().to_ascii_lowercase().as_str() {
        "pause" | "toggle_pause" | "togglepause" => Some(ControlCommand::TogglePause),
        "overlay" | "toggle_overlay" | "toggleoverlay" => Some(ControlCommand::ToggleOverlay),
        "control" | "toggle_control" | "togglecontrol" => Some(ControlCommand::ToggleControl),
        "performance" | "toggle_performance" | "toggleperformance" => {
            Some(ControlCommand::TogglePerformance)
        }
        "fps_hud" | "toggle_fps_hud" | "togglefpshud" | "fpshud" => {
            Some(ControlCommand::TogglePinnedFpsHud)
        }
        "notifications" | "toggle_notifications" | "togglenotifications" => {
            Some(ControlCommand::ToggleNotifications)
        }
        "reset" | "reset_game" | "resetgame" => Some(ControlCommand::ResetGame),
        "step" | "step_one_tick" | "steponetick" => Some(ControlCommand::StepOneTick),
        _ => None,
    }
}

pub fn parse_key_code(value: &str) -> Option<KeyCode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "p" | "keyp" => Some(KeyCode::KeyP),
        "esc" | "escape" => Some(KeyCode::Escape),
        "left" | "arrowleft" => Some(KeyCode::ArrowLeft),
        "right" | "arrowright" => Some(KeyCode::ArrowRight),
        "f1" => Some(KeyCode::F1),
        "f2" => Some(KeyCode::F2),
        "f3" => Some(KeyCode::F3),
        "f" | "keyf" => Some(KeyCode::KeyF),
        "r" | "keyr" => Some(KeyCode::KeyR),
        "period" | "." => Some(KeyCode::Period),
        _ => None,
    }
}

pub fn is_reserved_browser_key(key_code: KeyCode) -> bool {
    matches!(key_code, KeyCode::F12 | KeyCode::F11 | KeyCode::F10)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlPlaneAction {
    None,
    ResetRequested,
}

pub struct ControlPlaneState {
    simulation_paused: bool,
    pending_step_ticks: u32,
    hud_telemetry_mode: HudTelemetryMode,
    control_bindings: BTreeMap<KeyCode, ControlCommand>,
    overlay_visibility: OverlayVisibility,
    notifications: VecDeque<String>,
    max_notifications: usize,
}

impl ControlPlaneState {
    pub fn new(max_notifications: usize) -> Self {
        Self {
            simulation_paused: false,
            pending_step_ticks: 0,
            hud_telemetry_mode: HudTelemetryMode::Hidden,
            control_bindings: default_control_bindings(),
            overlay_visibility: OverlayVisibility::new(),
            notifications: VecDeque::new(),
            max_notifications,
        }
    }

    pub fn process_key_event(
        &mut self,
        key_code: KeyCode,
        is_pressed: bool,
        is_repeat: bool,
    ) -> Option<(ControlCommand, ControlPlaneAction)> {
        if !is_pressed || is_repeat {
            return None;
        }

        let command = self.control_bindings.get(&key_code).copied()?;
        let action = self.apply_command(command);
        Some((command, action))
    }

    pub fn apply_command(&mut self, command: ControlCommand) -> ControlPlaneAction {
        match command {
            ControlCommand::TogglePause => {
                if self.overlay_visibility.non_pause_overlay_visible() {
                    self.push_notification(
                        "Pause toggle suppressed while overlay context is active",
                    );
                    return ControlPlaneAction::None;
                }

                self.simulation_paused = !self.simulation_paused;
                self.overlay_visibility
                    .set_pause_menu(self.simulation_paused);
                if self.simulation_paused {
                    self.push_notification("Simulation paused");
                } else {
                    self.pending_step_ticks = 0;
                    self.push_notification("Simulation resumed");
                }
                self.sync_pause_scrim_visibility();
                ControlPlaneAction::None
            }
            ControlCommand::ToggleOverlay => {
                if self.overlay_visibility.control
                    || self.overlay_visibility.performance
                    || self.overlay_visibility.notifications
                {
                    self.overlay_visibility.control = false;
                    self.overlay_visibility.performance = false;
                    self.overlay_visibility.notifications = false;
                    self.push_notification("Overlay hidden");
                } else {
                    self.overlay_visibility.control = true;
                    self.overlay_visibility.performance = false;
                    self.overlay_visibility.notifications = false;
                    self.push_notification("Overlay opened: control");
                }
                self.sync_pause_scrim_visibility();
                ControlPlaneAction::None
            }
            ControlCommand::ToggleControl => {
                if !self.overlay_visibility.control {
                    let switched_tabs = self.overlay_visibility.performance;
                    self.overlay_visibility.control = true;
                    self.overlay_visibility.performance = false;
                    if switched_tabs {
                        self.push_notification("Overlay tab: control");
                    } else {
                        self.push_notification("Overlay opened: control");
                    }
                } else {
                    self.overlay_visibility.performance = false;
                    self.overlay_visibility.control = true;
                }
                self.sync_pause_scrim_visibility();
                ControlPlaneAction::None
            }
            ControlCommand::TogglePerformance => {
                if !self.overlay_visibility.performance {
                    let switched_tabs = self.overlay_visibility.control;
                    self.overlay_visibility.control = false;
                    self.overlay_visibility.performance = true;
                    if switched_tabs {
                        self.push_notification("Overlay tab: performance");
                    } else {
                        self.push_notification("Overlay opened: performance");
                    }
                } else {
                    self.overlay_visibility.control = false;
                    self.overlay_visibility.performance = true;
                }
                self.sync_pause_scrim_visibility();
                ControlPlaneAction::None
            }
            ControlCommand::TogglePinnedFpsHud => {
                self.hud_telemetry_mode = self.hud_telemetry_mode.next();
                self.push_notification(format!(
                    "HUD telemetry mode: {}",
                    self.hud_telemetry_mode.label()
                ));
                ControlPlaneAction::None
            }
            ControlCommand::ToggleNotifications => {
                if !self.overlay_visibility.notifications {
                    let switched_tabs =
                        self.overlay_visibility.control || self.overlay_visibility.performance;
                    self.overlay_visibility.control = false;
                    self.overlay_visibility.performance = false;
                    self.overlay_visibility.notifications = true;
                    if switched_tabs {
                        self.push_notification("Overlay tab: notifications");
                    } else {
                        self.push_notification("Overlay opened: notifications");
                    }
                } else {
                    self.overlay_visibility.control = false;
                    self.overlay_visibility.performance = false;
                    self.overlay_visibility.notifications = false;
                    self.push_notification("Overlay hidden");
                }
                self.sync_pause_scrim_visibility();
                ControlPlaneAction::None
            }
            ControlCommand::ResetGame => {
                self.push_notification("Reset requested");
                ControlPlaneAction::ResetRequested
            }
            ControlCommand::StepOneTick => {
                if self.simulation_paused {
                    self.pending_step_ticks = self.pending_step_ticks.saturating_add(1);
                    self.push_notification("Advanced one simulation tick");
                }
                ControlPlaneAction::None
            }
        }
    }

    pub fn set_control_binding(&mut self, command_name: &str, key_name: &str) -> bool {
        let Some(command) = parse_control_command(command_name) else {
            return false;
        };
        let Some(key_code) = parse_key_code(key_name) else {
            return false;
        };

        self.control_bindings.retain(|_, value| *value != command);
        self.control_bindings.insert(key_code, command);
        self.push_notification(format!(
            "Bound {} to {:?}",
            command_label(command),
            key_code
        ));
        true
    }

    pub fn should_run_fixed_tick(&mut self) -> bool {
        if self.is_overlay_pausing_simulation() {
            return false;
        }

        if self.simulation_paused {
            if self.pending_step_ticks > 0 {
                self.pending_step_ticks = self.pending_step_ticks.saturating_sub(1);
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn on_game_reset_complete(&mut self) {
        self.simulation_paused = false;
        self.pending_step_ticks = 0;
        self.hud_telemetry_mode = HudTelemetryMode::Hidden;
        self.overlay_visibility = OverlayVisibility::new();
        self.push_notification("Game reset");
    }

    pub fn on_controller_connected(&mut self, device_index: u32) {
        self.push_notification(format!("Controller {} connected", device_index));
    }

    pub fn on_controller_disconnected(&mut self, device_index: u32) {
        self.push_notification(format!("Controller {} disconnected", device_index));
    }

    pub fn is_simulation_paused(&self) -> bool {
        self.simulation_paused
    }

    pub fn is_runtime_paused(&self) -> bool {
        self.simulation_paused || self.is_overlay_pausing_simulation()
    }

    pub fn pending_step_ticks(&self) -> u32 {
        self.pending_step_ticks
    }

    pub fn input_context(&self) -> InputContext {
        if self.overlay_visibility.any_visible() {
            InputContext::OverlayContext
        } else {
            InputContext::GameplayContext
        }
    }

    pub fn allows_gameplay_input(&self) -> bool {
        self.input_context() == InputContext::GameplayContext
    }

    pub fn overlay_visibility(&self) -> OverlayVisibility {
        self.overlay_visibility
    }

    pub fn hud_telemetry_mode(&self) -> HudTelemetryMode {
        self.hud_telemetry_mode
    }

    pub fn is_fps_hud_pinned(&self) -> bool {
        self.hud_telemetry_mode.includes_fps()
    }

    pub fn notifications(&self) -> &VecDeque<String> {
        &self.notifications
    }

    fn is_overlay_pausing_simulation(&self) -> bool {
        self.overlay_visibility.control
            || self.overlay_visibility.performance
            || self.overlay_visibility.notifications
    }

    fn sync_pause_scrim_visibility(&mut self) {
        self.overlay_visibility.pause_menu =
            self.simulation_paused && !self.overlay_visibility.non_pause_overlay_visible();
    }

    fn push_notification(&mut self, message: impl Into<String>) {
        if self.notifications.len() >= self.max_notifications {
            self.notifications.pop_front();
        }
        self.notifications.push_back(message.into());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ControlCommand, ControlPlaneAction, ControlPlaneState, HudTelemetryMode, InputContext,
        command_label, default_control_bindings, parse_control_command, parse_key_code,
    };
    use winit::keyboard::KeyCode;

    #[test]
    fn control_command_parser_accepts_default_command_aliases() {
        assert_eq!(
            parse_control_command("pause"),
            Some(ControlCommand::TogglePause)
        );
        assert_eq!(
            parse_control_command("control"),
            Some(ControlCommand::ToggleControl)
        );
        assert_eq!(
            parse_control_command("toggle_performance"),
            Some(ControlCommand::TogglePerformance)
        );
        assert_eq!(
            parse_control_command("resetgame"),
            Some(ControlCommand::ResetGame)
        );
        assert_eq!(parse_control_command("unknown"), None);
    }

    #[test]
    fn key_code_parser_supports_default_control_keys() {
        assert_eq!(parse_key_code("p"), Some(KeyCode::KeyP));
        assert_eq!(parse_key_code("escape"), Some(KeyCode::Escape));
        assert_eq!(parse_key_code("left"), Some(KeyCode::ArrowLeft));
        assert_eq!(parse_key_code("right"), Some(KeyCode::ArrowRight));
        assert_eq!(parse_key_code("r"), Some(KeyCode::KeyR));
        assert_eq!(parse_key_code("?"), None);
    }

    #[test]
    fn default_control_bindings_include_required_defaults() {
        let bindings = default_control_bindings();
        assert_eq!(
            bindings.get(&KeyCode::KeyP),
            Some(&ControlCommand::TogglePause)
        );
        assert_eq!(
            bindings.get(&KeyCode::Escape),
            Some(&ControlCommand::ToggleOverlay)
        );
        assert_eq!(
            bindings.get(&KeyCode::ArrowLeft),
            Some(&ControlCommand::ToggleControl)
        );
        assert_eq!(
            bindings.get(&KeyCode::ArrowRight),
            Some(&ControlCommand::TogglePerformance)
        );
        assert_eq!(
            bindings.get(&KeyCode::F3),
            Some(&ControlCommand::ToggleNotifications)
        );
        assert_eq!(
            bindings.get(&KeyCode::KeyF),
            Some(&ControlCommand::TogglePinnedFpsHud)
        );
        assert_eq!(
            bindings.get(&KeyCode::KeyR),
            Some(&ControlCommand::ResetGame)
        );
    }

    #[test]
    fn command_label_is_stable() {
        assert_eq!(command_label(ControlCommand::TogglePause), "pause");
        assert_eq!(command_label(ControlCommand::ToggleOverlay), "overlay");
        assert_eq!(command_label(ControlCommand::ToggleControl), "control");
        assert_eq!(command_label(ControlCommand::TogglePinnedFpsHud), "fps_hud");
        assert_eq!(command_label(ControlCommand::ResetGame), "reset");
    }

    #[test]
    fn process_key_event_consumes_bound_command() {
        let mut state = ControlPlaneState::new(8);
        let consumed = state.process_key_event(KeyCode::KeyP, true, false);
        assert_eq!(
            consumed,
            Some((ControlCommand::TogglePause, ControlPlaneAction::None))
        );
        assert!(state.is_simulation_paused());
    }

    #[test]
    fn reset_command_emits_reset_action() {
        let mut state = ControlPlaneState::new(8);
        let consumed = state.process_key_event(KeyCode::KeyR, true, false);
        assert_eq!(
            consumed,
            Some((
                ControlCommand::ResetGame,
                ControlPlaneAction::ResetRequested
            ))
        );
    }

    #[test]
    fn fixed_tick_gate_blocks_when_paused_without_steps() {
        let mut state = ControlPlaneState::new(8);
        let _ = state.apply_command(ControlCommand::TogglePause);
        assert!(!state.should_run_fixed_tick());
    }

    #[test]
    fn fixed_tick_gate_allows_single_step_when_paused() {
        let mut state = ControlPlaneState::new(8);
        let _ = state.apply_command(ControlCommand::TogglePause);
        let _ = state.apply_command(ControlCommand::StepOneTick);
        assert!(state.should_run_fixed_tick());
        assert!(!state.should_run_fixed_tick());
    }

    #[test]
    fn fixed_tick_gate_blocks_when_overlay_visible() {
        let mut state = ControlPlaneState::new(8);
        let _ = state.apply_command(ControlCommand::ToggleControl);
        assert!(!state.should_run_fixed_tick());
    }

    #[test]
    fn runtime_pause_reports_overlay_state() {
        let mut state = ControlPlaneState::new(8);
        assert!(!state.is_runtime_paused());
        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        assert!(state.is_runtime_paused());
    }

    #[test]
    fn tab_switch_does_not_hide_overlay() {
        let mut state = ControlPlaneState::new(8);
        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        let _ = state.apply_command(ControlCommand::TogglePerformance);
        let overlays = state.overlay_visibility();
        assert!(overlays.performance);
        assert!(!overlays.control);

        let _ = state.apply_command(ControlCommand::ToggleControl);
        let overlays = state.overlay_visibility();
        assert!(overlays.control);
        assert!(!overlays.performance);
    }

    #[test]
    fn gameplay_context_is_default_when_no_overlay_visible() {
        let state = ControlPlaneState::new(8);
        assert_eq!(state.input_context(), InputContext::GameplayContext);
        assert!(state.allows_gameplay_input());
    }

    #[test]
    fn overlay_context_blocks_gameplay_input_when_overlay_visible() {
        let mut state = ControlPlaneState::new(8);
        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        assert_eq!(state.input_context(), InputContext::OverlayContext);
        assert!(!state.allows_gameplay_input());
    }

    #[test]
    fn pause_toggle_is_suppressed_while_non_pause_overlay_is_open() {
        let mut state = ControlPlaneState::new(8);
        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        let _ = state.apply_command(ControlCommand::ToggleControl);

        let _ = state.apply_command(ControlCommand::TogglePause);

        assert!(!state.is_simulation_paused());
        let overlays = state.overlay_visibility();
        assert!(overlays.control);
        assert!(!overlays.pause_menu);
    }

    #[test]
    fn pause_toggle_still_works_when_no_non_pause_overlay_is_open() {
        let mut state = ControlPlaneState::new(8);
        let _ = state.apply_command(ControlCommand::TogglePause);
        assert!(state.is_simulation_paused());

        let _ = state.apply_command(ControlCommand::TogglePause);
        assert!(!state.is_simulation_paused());
    }

    #[test]
    fn closing_overlay_restores_gameplay_context() {
        let mut state = ControlPlaneState::new(8);
        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        assert_eq!(state.input_context(), InputContext::OverlayContext);
        assert!(!state.allows_gameplay_input());

        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        assert_eq!(state.input_context(), InputContext::GameplayContext);
        assert!(state.allows_gameplay_input());
    }

    #[test]
    fn pause_toggle_recovers_after_overlay_context_closes() {
        let mut state = ControlPlaneState::new(8);

        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        let _ = state.apply_command(ControlCommand::TogglePerformance);
        let _ = state.apply_command(ControlCommand::TogglePause);
        assert!(!state.is_simulation_paused());

        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        let _ = state.apply_command(ControlCommand::TogglePause);
        assert!(state.is_simulation_paused());
    }

    #[test]
    fn opening_control_scrim_hides_pause_scrim() {
        let mut state = ControlPlaneState::new(8);

        let _ = state.apply_command(ControlCommand::TogglePause);
        assert!(state.overlay_visibility().pause_menu);

        let _ = state.apply_command(ControlCommand::ToggleControl);
        let overlays = state.overlay_visibility();
        assert!(overlays.control);
        assert!(!overlays.pause_menu);
    }

    #[test]
    fn closing_control_scrim_restores_pause_scrim_when_simulation_is_paused() {
        let mut state = ControlPlaneState::new(8);

        let _ = state.apply_command(ControlCommand::TogglePause);
        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        assert!(state.overlay_visibility().control);
        assert!(!state.overlay_visibility().pause_menu);

        let _ = state.apply_command(ControlCommand::ToggleOverlay);
        let overlays = state.overlay_visibility();
        assert!(!overlays.control);
        assert!(overlays.pause_menu);
    }

    #[test]
    fn pinned_fps_hud_toggles_independently_from_modal_overlays() {
        let mut state = ControlPlaneState::new(8);
        assert!(!state.is_fps_hud_pinned());
        assert_eq!(state.hud_telemetry_mode(), HudTelemetryMode::Hidden);

        let _ = state.apply_command(ControlCommand::TogglePinnedFpsHud);
        assert!(state.is_fps_hud_pinned());
        assert_eq!(state.hud_telemetry_mode(), HudTelemetryMode::Fps);
        assert_eq!(state.input_context(), InputContext::GameplayContext);

        let _ = state.apply_command(ControlCommand::TogglePinnedFpsHud);
        assert!(!state.is_fps_hud_pinned());
        assert_eq!(state.hud_telemetry_mode(), HudTelemetryMode::Hidden);
    }

    #[test]
    fn toggle_notifications_closes_notifications_overlay_when_pressed_twice() {
        let mut state = ControlPlaneState::new(8);

        let _ = state.apply_command(ControlCommand::ToggleNotifications);
        assert!(state.overlay_visibility().notifications);
        assert_eq!(state.input_context(), InputContext::OverlayContext);

        let _ = state.apply_command(ControlCommand::ToggleNotifications);
        assert!(!state.overlay_visibility().notifications);
        assert_eq!(state.input_context(), InputContext::GameplayContext);
    }
}
