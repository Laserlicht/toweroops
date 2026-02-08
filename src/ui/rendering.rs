use std::cell::RefCell;
use std::collections::HashMap;

use cairo::Context;
use gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;

use super::resources::{GameImage, GameResources};
use crate::game::field::BOARD_SIZE;
use crate::game::logic::GameState;
use crate::game::types::{CellKind, GameOutcome, Selection};

// Design-time (reference) dimensions – matches the original background bitmap size.
// All layout constants are relative to the (0,0) origin of this coordinate space.
pub const REF_WIDTH: f64 = 596.0;
pub const REF_HEIGHT: f64 = 393.0;

// Layout constants in reference coordinates
const FIELD_OFFSET_X: f64 = 136.0;
const FIELD_OFFSET_Y: f64 = 38.0;
const CELL_SIZE: f64 = 41.0;

const TOWER_LEFT_X: f64 = 31.0;
const TOWER_RIGHT_X: f64 = 501.0;
const TOWER_BASE_Y: f64 = 380.0;
const TOWER_ROW_HEIGHT: f64 = 16.0;
const TOWER_ROW_WIDTH_NORMAL: f64 = 68.0;
const TOWER_ROW_WIDTH_TOP: f64 = 84.0;

// Flags are SVG 31×31; place them centered above the tower top row.
const FLAG_SIZE: f64 = 31.0;
const FLAG_Y: f64 = TOWER_BASE_Y - TOWER_ROW_HEIGHT * 20.0 - FLAG_SIZE; // 29.0
                                                                        // Position so the flagpole (x=2 in the SVG) sits at the tower's horizontal center.
const FLAG_LEFT_X: f64 = 63.0; // center ~65 for left tower top width 84
const FLAG_RIGHT_X: f64 = 533.0; // center ~535 for right tower top width 84

// ── SVG rasterization cache ──────────────────────────────────────────────────
// Key: (pointer to usvg::Tree as usize, render_w_px, render_h_px)
// Value: pre-rasterized Pixbuf
// This avoids re-rasterizing SVGs every single frame and dramatically
// improves performance.  The cache is thread-local because GTK rendering
// happens on the main thread.
thread_local! {
    static SVG_CACHE: RefCell<HashMap<(usize, u32, u32), Pixbuf>> = RefCell::new(HashMap::new());
}

/// Render the entire game scene, scaled to fit (widget_w, widget_h).
/// `pulse_cell` = optional (col, row, progress 0..1) for the pulsing cell highlight.
/// `is_cpu_pulse` = true if the pulse is for the CPU move (red), false for player (blue).
pub fn render(
    cr: &Context,
    state: &GameState,
    res: &GameResources,
    widget_w: i32,
    widget_h: i32,
    anim_player_tower: f64,
    anim_computer_tower: f64,
    pulse_cell: Option<(usize, usize, f64)>,
    is_cpu_pulse: bool,
    raster_quality: f64,
) {
    let w = widget_w as f64;
    let h = widget_h as f64;
    let scale_x = w / REF_WIDTH;
    let scale_y = h / REF_HEIGHT;
    let scale = scale_x.min(scale_y);

    // Centre the scaled content
    let offset_x = (w - REF_WIDTH * scale) / 2.0;
    let offset_y = (h - REF_HEIGHT * scale) / 2.0;

    let _ = cr.save();
    cr.translate(offset_x, offset_y);
    cr.scale(scale, scale);

    // Draw background inside the same CTM so it uses the identical
    // translation/scale as the board and UI elements.
    if let Some(bg) = res.get("background") {
        draw_image_scaled(
            cr,
            bg,
            0.0,
            0.0,
            REF_WIDTH,
            REF_HEIGHT,
            scale,
            raster_quality,
        );
    }
    // Grid overlay (same size as background)
    if let Some(grid) = res.get("grid") {
        draw_image_scaled(
            cr,
            grid,
            0.0,
            0.0,
            REF_WIDTH,
            REF_HEIGHT,
            scale,
            raster_quality,
        );
    }

    // Draw the 8x8 board
    for col in 0..BOARD_SIZE {
        for row in 0..BOARD_SIZE {
            let cell = state.board.get(col, row);
            let x = FIELD_OFFSET_X + col as f64 * CELL_SIZE;
            let y = FIELD_OFFSET_Y + row as f64 * CELL_SIZE;

            let img = match cell.kind {
                CellKind::Bomb => res.bomb(cell.value),
                CellKind::Stone => res.stone(cell.value),
                CellKind::Banana => res.get("banana"),
                CellKind::Empty => None,
            };

            if let Some(img) = img {
                draw_image_scaled(cr, img, x, y, CELL_SIZE, CELL_SIZE, scale, raster_quality);
            }
        }
    }

    // Selection highlight (always visible)
    match state.selection {
        Selection::Column(c) => {
            if let Some(img) = res.get("vertical") {
                draw_image(
                    cr,
                    img,
                    FIELD_OFFSET_X - 1.0 + c as f64 * CELL_SIZE,
                    FIELD_OFFSET_Y - 1.0,
                    scale,
                    raster_quality,
                );
            }
        }
        Selection::Row(r) => {
            if let Some(img) = res.get("horizontal") {
                draw_image(
                    cr,
                    img,
                    FIELD_OFFSET_X - 1.0,
                    FIELD_OFFSET_Y - 1.0 + r as f64 * CELL_SIZE,
                    scale,
                    raster_quality,
                );
            }
        }
    }

    // Pulsing highlight on the selected cell
    if let Some((pc, pr, progress)) = pulse_cell {
        let px = FIELD_OFFSET_X + pc as f64 * CELL_SIZE;
        let py = FIELD_OFFSET_Y + pr as f64 * CELL_SIZE;
        draw_pulse_highlight(cr, px, py, CELL_SIZE, CELL_SIZE, progress, is_cpu_pulse);
    }

    // Hover highlight
    if let Some((hx, hy)) = state.hovered {
        if state.outcome == GameOutcome::Running {
            if let Some(img) = res.get("shadow") {
                draw_image(
                    cr,
                    img,
                    FIELD_OFFSET_X - 1.0 + hx as f64 * CELL_SIZE,
                    FIELD_OFFSET_Y - 1.0 + hy as f64 * CELL_SIZE,
                    scale,
                    raster_quality,
                );
            }
        }
    }

    // Tip
    if let Some((tx, ty)) = state.tip {
        if let Some(img) = res.get("tip") {
            draw_image(
                cr,
                img,
                FIELD_OFFSET_X - 1.0 + tx as f64 * CELL_SIZE,
                FIELD_OFFSET_Y - 1.0 + ty as f64 * CELL_SIZE,
                scale,
                raster_quality,
            );
        }
    }

    // Left tower (player) - uses animated height
    draw_tower(
        cr,
        res,
        anim_player_tower,
        TOWER_LEFT_X,
        scale,
        raster_quality,
    );
    // Right tower (computer)
    draw_tower(
        cr,
        res,
        anim_computer_tower,
        TOWER_RIGHT_X,
        scale,
        raster_quality,
    );

    // Flags
    if anim_player_tower >= 20.0 {
        if let Some(img) = res.get("flag_blue") {
            draw_image_scaled(
                cr,
                img,
                FLAG_LEFT_X,
                FLAG_Y,
                FLAG_SIZE,
                FLAG_SIZE,
                scale,
                raster_quality,
            );
        }
    }
    if anim_computer_tower >= 20.0 {
        if let Some(img) = res.get("flag_red") {
            draw_image_scaled(
                cr,
                img,
                FLAG_RIGHT_X,
                FLAG_Y,
                FLAG_SIZE,
                FLAG_SIZE,
                scale,
                raster_quality,
            );
        }
    }

    // Outcome overlay
    if state.outcome != GameOutcome::Running {
        let idx = match state.outcome {
            GameOutcome::Won => Some(0),
            GameOutcome::Lost => Some(1),
            GameOutcome::Drawn => Some(2),
            _ => None,
        };
        if let Some(idx) = idx {
            if let Some(img) = res.outcome_overlay(idx) {
                draw_image(cr, img, 0.0, 0.0, scale, raster_quality);
            }
        }
    }

    let _ = cr.restore();
}

fn draw_tower(
    cr: &Context,
    res: &GameResources,
    height: f64,
    base_x: f64,
    scale: f64,
    raster_quality: f64,
) {
    let int_height = height.floor() as i32;
    let frac = height - height.floor();

    for i in 0..20i32 {
        if i < int_height || (i == int_height && frac > 0.0) {
            let alpha = if i == int_height { frac } else { 1.0 };
            let (img, x_offset, target_w) = if i < 18 {
                (res.tower_row(i as usize % 2), 0.0, TOWER_ROW_WIDTH_NORMAL)
            } else if i == 18 {
                (res.tower_row(2), -8.0, TOWER_ROW_WIDTH_TOP)
            } else {
                (res.tower_row(3), -8.0, TOWER_ROW_WIDTH_TOP)
            };

            if let Some(img) = img {
                let x = base_x + x_offset;
                let y = TOWER_BASE_Y - TOWER_ROW_HEIGHT - i as f64 * TOWER_ROW_HEIGHT;
                if alpha >= 1.0 {
                    draw_image_scaled(
                        cr,
                        img,
                        x,
                        y,
                        target_w,
                        TOWER_ROW_HEIGHT,
                        scale,
                        raster_quality,
                    );
                } else {
                    draw_image_alpha_scaled(
                        cr,
                        img,
                        x,
                        y,
                        target_w,
                        TOWER_ROW_HEIGHT,
                        alpha,
                        scale,
                        raster_quality,
                    );
                }
            }
        }
    }
}

/// Convert widget-space mouse coordinates back to reference coordinates,
/// then to board (col, row).
pub fn mouse_to_cell(x: f64, y: f64, widget_w: i32, widget_h: i32) -> Option<(usize, usize)> {
    let w = widget_w as f64;
    let h = widget_h as f64;
    let scale_x = w / REF_WIDTH;
    let scale_y = h / REF_HEIGHT;
    let scale = scale_x.min(scale_y);
    let offset_x = (w - REF_WIDTH * scale) / 2.0;
    let offset_y = (h - REF_HEIGHT * scale) / 2.0;

    let rx = (x - offset_x) / scale;
    let ry = (y - offset_y) / scale;

    let col = ((rx - FIELD_OFFSET_X) / CELL_SIZE).floor() as i32;
    let row = ((ry - FIELD_OFFSET_Y) / CELL_SIZE).floor() as i32;

    if col >= 0 && col < BOARD_SIZE as i32 && row >= 0 && row < BOARD_SIZE as i32 {
        Some((col as usize, row as usize))
    } else {
        None
    }
}

// ── Image drawing helpers ────────────────────────────────────────────────────

/// Draw a GameImage (raster or SVG) at its native reference size.
fn draw_image(cr: &Context, img: &GameImage, x: f64, y: f64, scale: f64, raster_quality: f64) {
    match img {
        GameImage::Raster(pb) => {
            cr.set_source_pixbuf(pb, x, y);
            let _ = cr.paint();
        }
        GameImage::Svg { tree } => {
            let w = tree.size().width() as f64;
            let h = tree.size().height() as f64;
            render_svg(cr, tree, x, y, w, h, scale, 1.0, raster_quality);
        }
    }
}

/// Draw a GameImage scaled to fit within (target_w x target_h) in reference coords.
///
/// For raster images whose native pixel size differs from the target, Cairo
/// scaling is applied so the image fills exactly target_w × target_h.
fn draw_image_scaled(
    cr: &Context,
    img: &GameImage,
    x: f64,
    y: f64,
    target_w: f64,
    target_h: f64,
    scale: f64,
    raster_quality: f64,
) {
    match img {
        GameImage::Raster(pb) => {
            let pw = pb.width() as f64;
            let ph = pb.height() as f64;
            if pw <= 0.0 || ph <= 0.0 {
                return;
            }
            // Check if the native pixel size matches the target (within a pixel).
            // If so, just blit directly (the common case for game-piece PNGs).
            if (pw - target_w).abs() < 1.0 && (ph - target_h).abs() < 1.0 {
                cr.set_source_pixbuf(pb, x, y);
                let _ = cr.paint();
            } else {
                // Native size differs from target → scale (e.g. background 1248×832 → 564×420).
                let sx = target_w / pw;
                let sy = target_h / ph;
                let _ = cr.save();
                cr.translate(x, y);
                cr.scale(sx, sy);
                cr.set_source_pixbuf(pb, 0.0, 0.0);
                let _ = cr.paint();
                let _ = cr.restore();
            }
        }
        GameImage::Svg { tree } => {
            render_svg(
                cr,
                tree,
                x,
                y,
                target_w,
                target_h,
                scale,
                1.0,
                raster_quality,
            );
        }
    }
}

/// Draw a GameImage scaled to fit with alpha.
///
/// Like `draw_image_scaled` but with alpha support.
fn draw_image_alpha_scaled(
    cr: &Context,
    img: &GameImage,
    x: f64,
    y: f64,
    target_w: f64,
    target_h: f64,
    alpha: f64,
    scale: f64,
    raster_quality: f64,
) {
    match img {
        GameImage::Raster(pb) => {
            let pw = pb.width() as f64;
            let ph = pb.height() as f64;
            if pw <= 0.0 || ph <= 0.0 {
                return;
            }
            if (pw - target_w).abs() < 1.0 && (ph - target_h).abs() < 1.0 {
                cr.set_source_pixbuf(pb, x, y);
                let _ = cr.paint_with_alpha(alpha);
            } else {
                let sx = target_w / pw;
                let sy = target_h / ph;
                let _ = cr.save();
                cr.translate(x, y);
                cr.scale(sx, sy);
                cr.set_source_pixbuf(pb, 0.0, 0.0);
                let _ = cr.paint_with_alpha(alpha);
                let _ = cr.restore();
            }
        }
        GameImage::Svg { tree } => {
            render_svg(
                cr,
                tree,
                x,
                y,
                target_w,
                target_h,
                scale,
                alpha,
                raster_quality,
            );
        }
    }
}

/// Render an SVG tree onto a Cairo context at reference position (x, y)
/// with reference size (w x h).
///
/// The key insight: we rasterize the SVG at device-pixel resolution
/// (`w * scale`, `h * scale`) so the output is crisp, then temporarily
/// undo the CTM scaling to paint the pre-scaled pixbuf 1:1 onto the
/// device surface.  This avoids Cairo's bilinear upscaling of a small
/// raster image.
///
/// Results are cached so re-rasterization only happens when the target
/// pixel size changes (e.g. window resize), not every frame.
fn render_svg(
    cr: &Context,
    tree: &resvg::usvg::Tree,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    scale: f64,
    alpha: f64,
    raster_quality: f64,
) {
    let size = tree.size();
    let svg_w = size.width() as f64;
    let svg_h = size.height() as f64;
    if svg_w <= 0.0 || svg_h <= 0.0 || w <= 0.0 || h <= 0.0 || scale <= 0.0 {
        return;
    }

    let raster_quality = raster_quality.clamp(0.25, 1.0);
    let raster_scale = scale * raster_quality;

    // Target pixel dimensions = reference size x layout scale
    let render_w = (w * raster_scale).round().max(1.0) as u32;
    let render_h = (h * raster_scale).round().max(1.0) as u32;
    if render_w == 0 || render_h == 0 {
        return;
    }

    // Use the tree pointer + pixel dimensions as cache key
    let cache_key = (tree as *const _ as usize, render_w, render_h);

    // Try to get a cached pixbuf; if not found, rasterize and cache it
    let pixbuf = SVG_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(pb) = cache.get(&cache_key) {
            return pb.clone();
        }

        // Rasterize with resvg + tiny-skia at full device-pixel resolution
        let mut pixmap = match tiny_skia::Pixmap::new(render_w, render_h) {
            Some(pm) => pm,
            None => return Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, 1, 1).unwrap(),
        };

        let sx = render_w as f32 / size.width();
        let sy = render_h as f32 / size.height();
        let transform = tiny_skia::Transform::from_scale(sx, sy);
        resvg::render(tree, transform, &mut pixmap.as_mut());

        // Convert premultiplied RGBA -> straight RGBA and copy into an
        // owned Vec so the pixel data outlives the pixmap.
        let src = pixmap.data();
        let mut rgba = Vec::with_capacity(src.len());
        for chunk in src.chunks_exact(4) {
            let a = chunk[3] as u32;
            if a == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else if a < 255 {
                rgba.push(((chunk[0] as u32 * 255) / a).min(255) as u8);
                rgba.push(((chunk[1] as u32 * 255) / a).min(255) as u8);
                rgba.push(((chunk[2] as u32 * 255) / a).min(255) as u8);
                rgba.push(chunk[3]);
            } else {
                rgba.extend_from_slice(chunk);
            }
        }

        let pb = Pixbuf::from_mut_slice(
            rgba,
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            render_w as i32,
            render_h as i32,
            render_w as i32 * 4,
        );

        cache.insert(cache_key, pb.clone());
        pb
    });

    // Paint the pre-scaled pixbuf.  We temporarily undo the CTM scale
    // so the pixbuf pixels map 1:1 to device pixels.
    let _ = cr.save();
    // Move to the target position in reference coords, then undo the
    // uniform scale so we are in device-pixel space at the right location.
    cr.translate(x, y);
    cr.scale(1.0 / raster_scale, 1.0 / raster_scale);
    cr.set_source_pixbuf(&pixbuf, 0.0, 0.0);
    if alpha >= 1.0 {
        let _ = cr.paint();
    } else {
        let _ = cr.paint_with_alpha(alpha);
    }
    let _ = cr.restore();
}

/// Draw a pulsing coloured rectangle around a cell.
/// `progress` goes from 0.0 to 1.0 over the pulse duration.
/// The alpha and line width oscillate using a sine wave for a smooth pulse effect.
fn draw_pulse_highlight(cr: &Context, x: f64, y: f64, w: f64, h: f64, progress: f64, is_cpu: bool) {
    let t = (progress * 3.0 * 2.0 * std::f64::consts::PI).sin().abs();
    let alpha = 0.3 + 0.7 * t;
    let line_w = 2.0 + 2.0 * t;

    if is_cpu {
        cr.set_source_rgba(1.0, 0.2, 0.2, alpha);
    } else {
        cr.set_source_rgba(0.2, 0.5, 1.0, alpha);
    }
    cr.set_line_width(line_w);
    let inset = line_w / 2.0;
    cr.rectangle(x + inset, y + inset, w - line_w, h - line_w);
    let _ = cr.stroke();
}
