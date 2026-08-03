use std::time::{Duration, Instant};

pub struct RuntimeState {
    scheduler: FrameScheduler,
    last_frame_at: Option<Instant>,
    accumulated_time: Duration,
    frame_count: u64,
}

pub struct FrameScheduler {
    fixed_step: Duration,
    max_steps: u8,
    max_frame_delta: Duration,
}

pub struct FrameStepPlan {
    pub frame_index: u64,
    pub step_count: u32,
    pub accumulated_time: Duration,
    pub remaining_time: Duration,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            scheduler: FrameScheduler::new(),
            last_frame_at: None,
            accumulated_time: Duration::ZERO,
            frame_count: 0,
        }
    }

    pub fn begin_frame(&mut self, now: Instant) -> FrameStepPlan {
        let elapsed = self
            .last_frame_at
            .map(|last| now.saturating_duration_since(last))
            .unwrap_or_default();
        let clamped_elapsed = elapsed.min(self.scheduler.max_frame_delta);
        self.last_frame_at = Some(now);
        self.frame_count += 1;
        self.accumulated_time += clamped_elapsed;
        let plan = self
            .scheduler
            .plan_frame(self.frame_count, self.accumulated_time);
        self.accumulated_time = plan.remaining_time;
        plan
    }
}

impl FrameScheduler {
    pub fn new() -> Self {
        Self {
            fixed_step: Duration::from_millis(16),
            max_steps: 5,
            max_frame_delta: Duration::from_millis(250),
        }
    }

    pub fn plan_frame(&mut self, frame_index: u64, accumulated_time: Duration) -> FrameStepPlan {
        let clamped_time = accumulated_time.min(self.max_frame_delta);
        let step_count = (clamped_time.as_nanos() / self.fixed_step.as_nanos()) as u32;
        let step_count = step_count.min(self.max_steps as u32);
        let remaining_time = clamped_time.saturating_sub(self.fixed_step * step_count as u32);

        FrameStepPlan {
            frame_index,
            step_count,
            accumulated_time: clamped_time,
            remaining_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_plans_a_single_step_for_steady_delta() {
        let mut scheduler = FrameScheduler::new();
        let plan = scheduler.plan_frame(1, Duration::from_millis(16));

        assert_eq!(plan.step_count, 1);
        assert_eq!(plan.remaining_time, Duration::from_millis(0));
    }

    #[test]
    fn runtime_state_tracks_frame_progress() {
        let mut runtime = RuntimeState::new();
        let plan = runtime.begin_frame(Instant::now());

        assert_eq!(plan.step_count, 0);
        assert!(runtime.frame_count > 0);
    }

    #[test]
    fn runtime_state_clamps_large_frame_deltas() {
        let mut runtime = RuntimeState::new();
        let base = Instant::now();
        let _ = runtime.begin_frame(base);

        let plan = runtime.begin_frame(base + Duration::from_millis(500));

        assert_eq!(plan.step_count, 5);
        assert!(plan.remaining_time <= Duration::from_millis(250));
        assert!(runtime.accumulated_time <= Duration::from_millis(250));
    }

    #[test]
    fn runtime_state_carries_remainder_across_steps() {
        let mut runtime = RuntimeState::new();
        let base = Instant::now();
        let _ = runtime.begin_frame(base);

        let plan = runtime.begin_frame(base + Duration::from_millis(33));

        assert_eq!(plan.step_count, 2);
        assert!(plan.remaining_time <= Duration::from_millis(16));
        assert!(runtime.accumulated_time <= Duration::from_millis(16));
    }

    #[test]
    fn runtime_state_does_not_advance_for_zero_delta() {
        let mut runtime = RuntimeState::new();
        let base = Instant::now();
        let first = runtime.begin_frame(base);
        let second = runtime.begin_frame(base);

        assert_eq!(first.step_count, 0);
        assert_eq!(second.step_count, 0);
        assert_eq!(runtime.accumulated_time, Duration::ZERO);
    }

    #[test]
    fn frame_scheduler_caps_steps_for_large_accumulation() {
        let mut scheduler = FrameScheduler::new();
        scheduler.max_steps = 3;
        scheduler.max_frame_delta = Duration::from_secs(1);

        let plan = scheduler.plan_frame(1, Duration::from_millis(1000));

        assert_eq!(plan.step_count, 3);
        assert!(plan.remaining_time >= Duration::ZERO);
    }
}
