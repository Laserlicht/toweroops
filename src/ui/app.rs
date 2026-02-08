use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::gdk::Display;
use gtk4::gio::{Menu, SimpleAction};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, AspectFrame, Box as GtkBox, CssProvider, HeaderBar, Label,
    MenuButton, Orientation, Separator, STYLE_PROVIDER_PRIORITY_APPLICATION,
};

use super::board::{self, AnimationState};
use super::dialogs;
use super::resources::GameResources;
use crate::game::logic::GameState;
use crate::game::types::GameOutcome;
use crate::i18n::I18n;
use fluent_bundle::FluentArgs;

fn save_window_geometry(win: &ApplicationWindow, aspect_frame: Option<AspectFrame>) {
    let mut s = crate::storage::load_settings();
    let win_w = win.width();
    let win_h = win.height();

    // Default: if we don't have the aspect_frame, fall back to previous behavior.
    if aspect_frame.is_none() {
        // Keep sizes in the board aspect ratio so the background always fits.
        const BOARD_W: f64 = 596.0;
        const BOARD_H: f64 = 393.0;
        let aspect = BOARD_W / BOARD_H;
        let h_from_w = ((win_w as f64) / aspect).round() as i32;
        let w_from_h = ((win_h as f64) * aspect).round() as i32;
        let (final_w, final_h) = {
            let dw = (win_h as i32 - h_from_w).abs();
            let dh = (win_w as i32 - w_from_h).abs();
            if dw <= dh {
                (win_w, h_from_w)
            } else {
                (w_from_h, win_h)
            }
        };
        s.window_width = Some(final_w);
        s.window_height = Some(final_h);
        let _ = crate::storage::save_settings(&s);
        return;
    }

    // If we have the aspect_frame, measure the difference between window and board
    // content. Use that delta to compute a window size that results in a board
    // size matching the board aspect ratio exactly.
    let af = aspect_frame.unwrap();
    let af_w = af.width();
    let af_h = af.height();

    let delta_w = win_w - af_w;
    let delta_h = win_h - af_h;

    const BOARD_W: f64 = 596.0;
    const BOARD_H: f64 = 393.0;
    let aspect = BOARD_W / BOARD_H;

    // Determine target board size by snapping either width->height or height->width
    // and choosing the one closest to current board size.
    let h_from_w = ((af_w as f64) / aspect).round() as i32;
    let w_from_h = ((af_h as f64) * aspect).round() as i32;
    let (final_board_w, final_board_h) = {
        let dw = (af_h as i32 - h_from_w).abs();
        let dh = (af_w as i32 - w_from_h).abs();
        if dw <= dh {
            (af_w, h_from_w)
        } else {
            (w_from_h, af_h)
        }
    };

    // Add deltas back to get the window size that will contain the board
    let final_win_w = final_board_w + delta_w;
    let final_win_h = final_board_h + delta_h;

    s.window_width = Some(final_win_w);
    s.window_height = Some(final_win_h);
    let _ = crate::storage::save_settings(&s);
}

/// Build and present the main application window.
pub fn build_ui(app: &Application, resources_dir: &str) {
    // ── Shared state ──
    // Load persisted settings and statistics (if present) and apply to initial state.
    let settings = crate::storage::load_settings();
    let mut initial_state = GameState::new();
    initial_state.ai_level = settings.ai_level;
    initial_state.statistics = crate::storage::load_statistics();
    let state = Rc::new(RefCell::new(initial_state));
    let resources = Rc::new(GameResources::load(resources_dir));
    let i18n = Rc::new(I18n::load_from_dir(resources_dir));
    let anim = Rc::new(RefCell::new(AnimationState::new()));
    // Apply persisted animation speed (convert legacy "per-tick" values to rows/sec)
    {
        let mut an = anim.borrow_mut();
        let mut speed = settings.animation_speed;
        if speed > 0.0 && speed < 1.0 {
            // Old semantics were "rows per frame" (~60fps); convert to rows per second.
            speed *= 60.0;
        }
        if speed <= 0.0 {
            speed = AnimationState::new().speed;
        }
        an.speed = speed;
    }

    // ── CSS ──
    let provider = CssProvider::new();
    let css = "
        .title-label  { font-weight: 700; font-size: 15px; }
        .stat-label   { font-size: 12px; margin: 0 6px; }
        .game-board   { background-color: #2d2d2d; }
    ";
    provider.load_from_data(css);
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // ── Window ──
    let win_title = i18n.t("app-title");
    let window = ApplicationWindow::builder()
        .application(app)
        .title(&win_title)
        .default_width(596)
        .default_height(440)
        .resizable(true)
        .build();

    // Apply persisted window size (if present)
    if let (Some(w), Some(h)) = (settings.window_width, settings.window_height) {
        window.set_default_size(w, h);
    }

    // ── Header bar ──
    let header = HeaderBar::new();
    header.set_show_title_buttons(true);
    let header_title = Label::new(Some(&i18n.t("app-title")));
    header_title.add_css_class("title-label");
    header.set_title_widget(Some(&header_title));

    // ── Hamburger menu ──
    let menu = Menu::new();
    menu.append(Some(&i18n.t("menu-new-game")), Some("win.new-game"));
    menu.append(
        Some(&i18n.t("menu-computer-begins")),
        Some("win.computer-begins"),
    );
    menu.append(Some(&i18n.t("menu-hint")), Some("win.hint"));

    let section2 = Menu::new();
    section2.append(Some(&i18n.t("menu-settings")), Some("win.settings"));
    section2.append(Some(&i18n.t("menu-info")), Some("win.info"));
    menu.append_section(None, &section2);

    let menu_button = MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    menu_button.set_menu_model(Some(&menu));
    header.pack_end(&menu_button);

    // ── Main layout ──
    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Game board – wrapped in an AspectFrame to keep the background's aspect ratio
    let drawing_area = board::create_board(state.clone(), resources.clone(), anim.clone());
    drawing_area.add_css_class("game-board");
    let aspect_frame = AspectFrame::new(0.5, 0.5, 596.0 / 393.0, false);
    aspect_frame.set_child(Some(&drawing_area));
    aspect_frame.set_hexpand(true);
    aspect_frame.set_vexpand(true);
    main_box.append(&aspect_frame);

    // Status bar
    let status_bar = GtkBox::new(Orientation::Horizontal, 8);
    status_bar.set_margin_start(8);
    status_bar.set_margin_end(8);
    status_bar.set_margin_top(4);
    status_bar.set_margin_bottom(4);

    let stat_player = Label::new(None);
    stat_player.add_css_class("stat-label");
    let stat_computer = Label::new(None);
    stat_computer.add_css_class("stat-label");
    let stat_drawn = Label::new(None);
    stat_drawn.add_css_class("stat-label");

    status_bar.append(&stat_player);
    status_bar.append(&Separator::new(Orientation::Vertical));
    status_bar.append(&stat_computer);
    status_bar.append(&Separator::new(Orientation::Vertical));
    status_bar.append(&stat_drawn);

    main_box.append(&status_bar);

    // ── Stats updater ──
    let update_stats = {
        let state = state.clone();
        let i18n = i18n.clone();
        let stat_player = stat_player.clone();
        let stat_computer = stat_computer.clone();
        let stat_drawn = stat_drawn.clone();
        move || {
            let st = state.borrow();
            stat_player.set_text(&format!(
                "{}: {}",
                i18n.t("stat-player"),
                st.statistics.player_wins
            ));
            stat_computer.set_text(&format!(
                "{}: {}",
                i18n.t("stat-computer"),
                st.statistics.computer_wins
            ));
            stat_drawn.set_text(&format!(
                "{}: {}",
                i18n.t("stat-drawn"),
                st.statistics.draws
            ));
        }
    };
    update_stats();

    // ── Animation tick (time-based) ──
    {
        let state = state.clone();
        let anim = anim.clone();
        let drawing_area = drawing_area.clone();
        let update_stats = update_stats.clone();
        let last_time = Rc::new(RefCell::new(Instant::now()));
        drawing_area.add_tick_callback(move |widget, _clock| {
            let now = Instant::now();
            let mut lt = last_time.borrow_mut();
            let dt = now.duration_since(*lt).as_secs_f64();
            *lt = now;

            let st = state.borrow();
            let target_p = st.tower_player as f64;
            let target_c = st.tower_computer as f64;
            drop(st);

            let mut an = anim.borrow_mut();
            let mut need_redraw = an.tick_towers(target_p, target_c, dt);

            // Drive the animation state machine
            match an.phase.clone() {
                board::AnimPhase::Idle => {}

                board::AnimPhase::PlayerPulse {
                    col,
                    row,
                    time_left,
                    total,
                } => {
                    need_redraw = true;
                    if time_left <= Duration::from_secs(0) {
                        // Pulse done → apply the player's move
                        an.phase = board::AnimPhase::Idle;
                        drop(an);
                        let mut st = state.borrow_mut();
                        let result = st.make_move(col, row, true);
                        if result == crate::game::logic::MoveResult::Continue {
                            // Game continues → schedule CPU turn
                            drop(st);
                            let mut an = anim.borrow_mut();
                            let wait = an.wait_before_cpu_duration();
                            an.phase = board::AnimPhase::WaitBeforeCpu { time_left: wait };
                        }
                    } else {
                        let remaining = time_left.saturating_sub(Duration::from_secs_f64(dt));
                        an.phase = board::AnimPhase::PlayerPulse {
                            col,
                            row,
                            time_left: remaining,
                            total,
                        };
                    }
                }

                board::AnimPhase::WaitBeforeCpu { time_left } => {
                    need_redraw = true;
                    if time_left <= Duration::from_secs(0) {
                        // Pause done → CPU picks a move and starts pulsing
                        drop(an);
                        let st = state.borrow();
                        if st.outcome == GameOutcome::Running {
                            let (col, row) = st.compute_ai_move();
                            drop(st);
                            let mut an = anim.borrow_mut();
                            let dur = an.pulse_duration();
                            an.phase = board::AnimPhase::CpuPulse {
                                col,
                                row,
                                time_left: dur,
                                total: dur,
                            };
                        } else {
                            drop(st);
                            anim.borrow_mut().phase = board::AnimPhase::Idle;
                        }
                    } else {
                        let remaining = time_left.saturating_sub(Duration::from_secs_f64(dt));
                        an.phase = board::AnimPhase::WaitBeforeCpu {
                            time_left: remaining,
                        };
                    }
                }

                board::AnimPhase::CpuPulse {
                    col,
                    row,
                    time_left,
                    total,
                } => {
                    need_redraw = true;
                    if time_left <= Duration::from_secs(0) {
                        // Pulse done → apply the CPU's move
                        an.phase = board::AnimPhase::Idle;
                        drop(an);
                        let mut st = state.borrow_mut();
                        st.make_move(col, row, false);
                    } else {
                        let remaining = time_left.saturating_sub(Duration::from_secs_f64(dt));
                        an.phase = board::AnimPhase::CpuPulse {
                            col,
                            row,
                            time_left: remaining,
                            total,
                        };
                    }
                }
            }

            if need_redraw {
                widget.queue_draw();
            }
            update_stats();
            glib::Continue(true)
        });
    }

    // ── Actions ──
    // New Game
    {
        let action = SimpleAction::new("new-game", None);
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let update_stats = update_stats.clone();
        let i18n = i18n.clone();
        let win_for_closure = window.clone();
        let anim = anim.clone();
        action.connect_activate(move |_, _| {
            let running = state.borrow().outcome == GameOutcome::Running;
            if running && state.borrow().moves_made > 0 {
                let state = state.clone();
                let drawing_area = drawing_area.clone();
                let update_stats = update_stats.clone();
                let anim = anim.clone();
                dialogs::confirm_surrender(&win_for_closure, &i18n, move || {
                    let mut st = state.borrow_mut();
                    st.surrender();
                    st.new_game();
                    anim.borrow_mut().snap(0.0, 0.0);
                    drop(st);
                    drawing_area.queue_draw();
                    update_stats();
                });
            } else {
                state.borrow_mut().new_game();
                anim.borrow_mut().snap(0.0, 0.0);
                drawing_area.queue_draw();
                update_stats();
            }
        });
        window.add_action(&action);
    }

    // Computer begins
    {
        let action = SimpleAction::new("computer-begins", None);
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let anim = anim.clone();
        action.connect_activate(move |_, _| {
            let st = state.borrow();
            if st.moves_made == 0 && st.outcome == GameOutcome::Running {
                let (col, row) = st.compute_ai_move();
                drop(st);
                let mut an = anim.borrow_mut();
                if !an.is_busy() {
                    let dur = an.pulse_duration();
                    an.phase = board::AnimPhase::CpuPulse {
                        col,
                        row,
                        time_left: dur,
                        total: dur,
                    };
                }
                drawing_area.queue_draw();
            }
        });
        window.add_action(&action);
    }

    // Hint
    {
        let action = SimpleAction::new("hint", None);
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        action.connect_activate(move |_, _| {
            state.borrow_mut().get_tip();
            drawing_area.queue_draw();
        });
        window.add_action(&action);
    }

    // Settings
    {
        let action = SimpleAction::new("settings", None);
        let state = state.clone();
        let anim = anim.clone();
        let i18n = i18n.clone();
        let win_for_closure = window.clone();
        action.connect_activate(move |_, _| {
            dialogs::show_settings_dialog(&win_for_closure, state.clone(), anim.clone(), &i18n);
        });
        window.add_action(&action);
    }

    // Info
    {
        let action = SimpleAction::new("info", None);
        let win_for_closure = window.clone();
        let i18n = i18n.clone();
        action.connect_activate(move |_, _| {
            let mut args = FluentArgs::new();
            args.set("version", env!("CARGO_PKG_VERSION"));
            let mut body = i18n.t_args("info-body", &args);
            // Fluent stores literal "\n" sequences; convert them to real newlines
            body = body.replace("\\n", "\n");
            // Append localized info link (may contain markup)
            let link = i18n.t("info-link");
            body.push_str("\n\n");
            body.push_str(&link);
            dialogs::show_info(&win_for_closure, &i18n.t("menu-info"), &body, &i18n);
        });
        window.add_action(&action);
    }

    // ── Close-request handler (warn if game in progress) ──
    {
        let state = state.clone();
        let i18n = i18n.clone();
        // clone the aspect_frame so we can reference it from closures below
        let aspect_frame_for_save = aspect_frame.clone();
        window.connect_close_request(move |win| {
            let st = state.borrow();
            if st.outcome == GameOutcome::Running && st.moves_made > 0 {
                drop(st);
                let dialog = dialogs::confirm_close(win, &i18n);
                let win = win.clone();
                dialog.connect_response(move |dialog, response| {
                    dialog.close();
                    if response == gtk4::ResponseType::Accept {
                        // On accept: destroy the window. `connect_destroy` will save geometry.
                        win.destroy();
                    }
                });
                dialog.show();
                gtk4::Inhibit(true)
            } else {
                // Save geometry before allowing the window to close
                save_window_geometry(win, Some(aspect_frame_for_save.clone()));
                gtk4::Inhibit(false)
            }
        });
    }

    window.set_titlebar(Some(&header));
    window.set_child(Some(&main_box));

    // Persist window size when the window is destroyed so it can be
    // restored on next startup. Ignore save errors.
    {
        let aspect_frame_for_save = aspect_frame.clone();
        window.connect_destroy(move |win| {
            save_window_geometry(win, Some(aspect_frame_for_save.clone()));
        });
    }

    window.present();
}
