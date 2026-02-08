use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{DrawingArea, EventControllerMotion, GestureClick};

use super::rendering;
use super::resources::GameResources;
use crate::game::logic::GameState;
use crate::game::types::GameOutcome;

/// The phases of the turn animation state machine.
#[derive(Debug, Clone)]
pub enum AnimPhase {
    /// Waiting for the player to click.
    Idle,
    /// Player clicked (col, row) – pulsing highlight for a fixed duration, then apply.
    PlayerPulse {
        col: usize,
        row: usize,
        time_left: Duration,
        total: Duration,
    },
    /// Player move was applied. Short pause before CPU thinks.
    WaitBeforeCpu { time_left: Duration },
    /// CPU has chosen (col, row) – pulsing highlight for a fixed duration, then apply.
    CpuPulse {
        col: usize,
        row: usize,
        time_left: Duration,
        total: Duration,
    },
}

/// Animation state: tower interpolation + turn phase machine.
pub struct AnimationState {
    pub display_player_tower: f64,
    pub display_computer_tower: f64,
    /// Tower animation speed: rows per second. Default 12.0 (≈0.2 per 60fps tick).
    pub speed: f64,
    /// Current phase.
    pub phase: AnimPhase,
}

/// Pulse duration.
const PULSE_DURATION: Duration = Duration::from_millis(400);
/// Pause before the CPU acts.
const WAIT_BEFORE_CPU_DURATION: Duration = Duration::from_millis(160);
const RESIZE_INTERPOLATION_MS: u64 = 500;
const RESIZE_LOW_QUALITY: f64 = 0.6;

struct ResizeState {
    last_size: (i32, i32),
    last_change: Instant,
    generation: u64,
}

impl ResizeState {
    fn new() -> Self {
        Self {
            last_size: (0, 0),
            last_change: Instant::now(),
            generation: 0,
        }
    }
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            display_player_tower: 0.0,
            display_computer_tower: 0.0,
            speed: 12.0,
            phase: AnimPhase::Idle,
        }
    }

    /// Is a pulse/wait animation running? (blocks clicks)
    pub fn is_busy(&self) -> bool {
        !matches!(self.phase, AnimPhase::Idle)
    }

    /// Get the currently pulsing cell (if any) and its progress 0.0..1.0.
    pub fn pulse_cell(&self) -> Option<(usize, usize, f64)> {
        match &self.phase {
            AnimPhase::PlayerPulse {
                col,
                row,
                time_left,
                total,
            } => {
                let elapsed = (*total - *time_left).as_secs_f64();
                Some((*col, *row, (elapsed / total.as_secs_f64()).clamp(0.0, 1.0)))
            }
            AnimPhase::CpuPulse {
                col,
                row,
                time_left,
                total,
            } => {
                let elapsed = (*total - *time_left).as_secs_f64();
                Some((*col, *row, (elapsed / total.as_secs_f64()).clamp(0.0, 1.0)))
            }
            _ => None,
        }
    }

    /// Is the current pulse for the CPU?
    pub fn is_cpu_pulse(&self) -> bool {
        matches!(self.phase, AnimPhase::CpuPulse { .. })
    }

    /// Step toward the target tower values. Returns `true` if still animating.
    /// Advance tower interpolation by `dt` seconds.
    pub fn tick_towers(&mut self, target_player: f64, target_computer: f64, dt: f64) -> bool {
        let mut changed = false;
        changed |= Self::step(
            &mut self.display_player_tower,
            target_player,
            self.speed,
            dt,
        );
        changed |= Self::step(
            &mut self.display_computer_tower,
            target_computer,
            self.speed,
            dt,
        );
        changed
    }

    fn step(current: &mut f64, target: f64, speed: f64, dt: f64) -> bool {
        let diff = target - *current;
        if diff.abs() < 0.01 {
            if (*current - target).abs() > f64::EPSILON {
                *current = target;
                return true;
            }
            return false;
        }
        let step = speed * dt;
        *current += diff.signum() * step.min(diff.abs());
        true
    }

    /// Instantly snap towers to target (e.g. on new game).
    pub fn snap(&mut self, player: f64, computer: f64) {
        self.display_player_tower = player;
        self.display_computer_tower = computer;
        self.phase = AnimPhase::Idle;
    }

    /// Pulse duration used for player/CPU pulse.
    pub fn pulse_duration(&self) -> Duration {
        PULSE_DURATION
    }

    /// Wait duration before the CPU acts.
    pub fn wait_before_cpu_duration(&self) -> Duration {
        WAIT_BEFORE_CPU_DURATION
    }
}

/// Create the game board drawing area widget with mouse handling.
pub fn create_board(
    state: Rc<RefCell<GameState>>,
    resources: Rc<GameResources>,
    anim: Rc<RefCell<AnimationState>>,
) -> DrawingArea {
    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(596);
    drawing_area.set_content_height(393);
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    let resize_state = Rc::new(RefCell::new(ResizeState::new()));

    // --- Draw handler ---
    {
        let state = state.clone();
        let resources = resources.clone();
        let anim = anim.clone();
        let resize_state = resize_state.clone();
        drawing_area.set_draw_func(move |area, cr, w, h| {
            let now = Instant::now();
            let mut rs = resize_state.borrow_mut();
            if rs.last_size != (w, h) {
                rs.last_size = (w, h);
                rs.last_change = now;
                rs.generation = rs.generation.wrapping_add(1);
                let gen = rs.generation;
                let da = area.clone();
                let resize_state = resize_state.clone();
                let delay = Duration::from_millis(RESIZE_INTERPOLATION_MS);
                glib::timeout_add_local_once(delay, move || {
                    let current = resize_state.borrow().generation;
                    if current == gen {
                        da.queue_draw();
                    }
                });
            }
            let elapsed = now.duration_since(rs.last_change);
            let raster_quality = if elapsed < Duration::from_millis(RESIZE_INTERPOLATION_MS) {
                RESIZE_LOW_QUALITY
            } else {
                1.0
            };
            drop(rs);

            let st = state.borrow();
            let an = anim.borrow();
            rendering::render(
                cr,
                &st,
                &resources,
                w,
                h,
                an.display_player_tower,
                an.display_computer_tower,
                an.pulse_cell(),
                an.is_cpu_pulse(),
                raster_quality,
            );
        });
    }

    // --- Click handler ---
    {
        let state = state.clone();
        let da = drawing_area.clone();
        let anim = anim.clone();
        let click = GestureClick::new();
        click.connect_released(move |_gesture, _n, x, y| {
            // Ignore clicks while animation is busy
            if anim.borrow().is_busy() {
                return;
            }
            let w = da.width();
            let h = da.height();
            if let Some((col, row)) = rendering::mouse_to_cell(x, y, w, h) {
                let st = state.borrow();
                if st.outcome != GameOutcome::Running {
                    return;
                }
                if !st.is_valid_move(col, row) {
                    return;
                }
                drop(st);
                // Start player pulse animation (time based)
                let mut an = anim.borrow_mut();
                let dur = an.pulse_duration();
                an.phase = AnimPhase::PlayerPulse {
                    col,
                    row,
                    time_left: dur,
                    total: dur,
                };
                da.queue_draw();
            }
        });
        drawing_area.add_controller(click);
    }

    // --- Mouse move handler ---
    {
        let state = state.clone();
        let da = drawing_area.clone();
        let motion = EventControllerMotion::new();
        motion.connect_motion(move |_ctrl, x, y| {
            let w = da.width();
            let h = da.height();
            let mut st = state.borrow_mut();
            if let Some((col, row)) = rendering::mouse_to_cell(x, y, w, h) {
                st.update_hover(col, row);
            } else {
                st.clear_hover();
            }
            drop(st);
            da.queue_draw();
        });
        drawing_area.add_controller(motion);
    }

    drawing_area
}
