use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{Adjustment, ApplicationWindow, Dialog, Label, ResponseType, Scale, Switch};

use super::board::AnimationState;
use crate::game::logic::GameState;
use crate::i18n::I18n;

/// Show a settings dialog (AI level, animation speed, reset statistics).
pub fn show_settings_dialog(
    parent: &ApplicationWindow,
    state: Rc<RefCell<GameState>>,
    anim: Rc<RefCell<AnimationState>>,
    i18n: &I18n,
) {
    let dialog = Dialog::new();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_destroy_with_parent(true);
    dialog.set_title(Some(&i18n.t("settings-title")));
    dialog.set_default_width(380);

    // Create buttons and add spacing around them
    let ok_btn = dialog.add_button(&i18n.t("ok"), ResponseType::Accept);
    let cancel_btn = dialog.add_button(&i18n.t("cancel"), ResponseType::Cancel);
    ok_btn.set_margin_start(8);
    ok_btn.set_margin_end(8);
    ok_btn.set_margin_top(6);
    ok_btn.set_margin_bottom(6);
    cancel_btn.set_margin_start(8);
    cancel_btn.set_margin_end(8);
    cancel_btn.set_margin_top(6);
    cancel_btn.set_margin_bottom(6);

    let content = dialog.content_area();
    content.set_spacing(12);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    // ── AI level ──
    let level_label = Label::new(Some(&format!(
        "{}: {}",
        i18n.t("settings-level"),
        state.borrow().ai_level
    )));
    content.append(&level_label);

    let level_adj = Adjustment::new(
        state.borrow().ai_level as f64,
        0.0,
        crate::ai::MAX_AI_LEVEL as f64,
        1.0,
        1.0,
        0.0,
    );
    let level_scale = Scale::new(gtk4::Orientation::Horizontal, Some(&level_adj));
    level_scale.set_digits(0);
    level_scale.set_hexpand(true);
    // Add marks for each level
    for i in 0..=crate::ai::MAX_AI_LEVEL {
        level_scale.add_mark(i as f64, gtk4::PositionType::Bottom, Some(&i.to_string()));
    }
    content.append(&level_scale);

    {
        let level_label = level_label.clone();
        let key = i18n.t("settings-level");
        level_adj.connect_value_changed(move |adj| {
            level_label.set_text(&format!("{}: {}", key, adj.value() as i32));
        });
    }

    // ── Reset statistics ──
    let reset_switch = Switch::new();
    reset_switch.set_active(false);
    let reset_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let reset_label = Label::new(Some(&i18n.t("settings-reset")));
    reset_box.append(&reset_label);
    reset_box.append(&reset_switch);
    content.append(&reset_box);

    let state_clone = state.clone();
    let reset_switch_clone = reset_switch.clone();
    let anim_clone = anim.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let mut st = state_clone.borrow_mut();
            st.ai_level = level_adj.value() as i32;
            if reset_switch_clone.is_active() {
                st.statistics.reset();
                let _ = crate::storage::save_statistics(&st.statistics);
            }

            // Persist updated settings (ai level + animation speed)
            let current_anim_speed = anim_clone.borrow().speed;
            let mut settings = crate::storage::load_settings();
            settings.ai_level = st.ai_level;
            settings.animation_speed = current_anim_speed;
            let _ = crate::storage::save_settings(&settings);
        }
        dialog.close();
    });

    dialog.show();
}

/// Show a "surrender?" confirmation dialog.
pub fn confirm_surrender(parent: &ApplicationWindow, i18n: &I18n, on_confirm: impl Fn() + 'static) {
    let dialog = Dialog::with_buttons(
        Some(&i18n.t("surrender-title")),
        Some(parent),
        gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
        &[
            (&i18n.t("ok"), ResponseType::Accept),
            (&i18n.t("cancel"), ResponseType::Cancel),
        ],
    );

    let content = dialog.content_area();
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    let label = Label::new(Some(&i18n.t("surrender-message")));
    label.set_wrap(true);
    content.append(&label);

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            on_confirm();
        }
        dialog.close();
    });

    dialog.show();
}

/// Show a simple info message box.
pub fn show_info(parent: &ApplicationWindow, title: &str, message: &str, i18n: &I18n) {
    let dialog = Dialog::new();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_destroy_with_parent(true);
    dialog.set_title(Some(title));
    let ok_btn = dialog.add_button(&i18n.t("ok"), ResponseType::Accept);
    ok_btn.set_margin_start(8);
    ok_btn.set_margin_end(8);
    ok_btn.set_margin_top(6);
    ok_btn.set_margin_bottom(6);

    let content = dialog.content_area();
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    // Use markup so links can be shown as anchors. Connect the activate-link
    // signal to open clicked URIs in the user's default browser.
    let label = Label::new(None);
    label.set_wrap(true);
    label.set_use_markup(true);
    label.set_markup(message);
    // Open links when activated
    label.connect_activate_link(|_, uri| {
        if let Err(e) = gio::AppInfo::launch_default_for_uri(uri, None::<&gio::AppLaunchContext>) {
            eprintln!("Failed to open link {}: {}", uri, e);
            return gtk4::Inhibit(false);
        }
        gtk4::Inhibit(true)
    });
    content.append(&label);

    dialog.connect_response(|dialog, _| {
        dialog.close();
    });

    // ensure some space around the OK button
    // (button margins set above)

    dialog.show();
}

/// Show a "quit while game running?" confirmation. Returns a Dialog the caller
/// can wait on, or use the callback approach.
pub fn confirm_close(parent: &ApplicationWindow, i18n: &I18n) -> Dialog {
    let dialog = Dialog::new();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_destroy_with_parent(true);
    dialog.set_title(Some(&i18n.t("close-confirm-title")));

    // Add buttons and ensure some space around them
    let ok_btn = dialog.add_button(&i18n.t("ok"), ResponseType::Accept);
    let cancel_btn = dialog.add_button(&i18n.t("cancel"), ResponseType::Cancel);
    ok_btn.set_margin_start(8);
    ok_btn.set_margin_end(8);
    ok_btn.set_margin_top(6);
    ok_btn.set_margin_bottom(6);
    cancel_btn.set_margin_start(8);
    cancel_btn.set_margin_end(8);
    cancel_btn.set_margin_top(6);
    cancel_btn.set_margin_bottom(6);

    let content = dialog.content_area();
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    let label = Label::new(Some(&i18n.t("close-confirm-message")));
    label.set_wrap(true);
    content.append(&label);

    dialog
}
