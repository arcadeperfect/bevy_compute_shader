/*
Adapted from: https://gitlab.com/polwel/egui-colorgradient/-/tree/master

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
*/


//! Gradient editor widget for [egui](https://www.egui.rs/).

use bevy_egui::egui::color_picker::{color_picker_hsva_2d, Alpha};
use bevy_egui::egui::ecolor::Hsva;
use bevy_egui::egui::style::WidgetVisuals;
use bevy_egui::egui::{
    lerp, pos2, vec2, Area, Button, Color32, ColorImage, ComboBox, Frame, Id, Key, LayerId, Mesh,
    Order, Painter, PointerButton, Rect, Sense, Shape, Stroke, TextureHandle, TextureOptions, Ui,
    Vec2,
};

use super::cache::FrameCacheDyn;
pub use super::gradient::{ColorInterpolator, Gradient, InterpolationMethod};

// mod cache;
// mod gradient;
const TICK_OFFSET: f32 = 8.;

const TICK_SQUARE_SIZE: f32 = 12.;

const CHECKER_SIZE: f32 = 15.0;

fn background_checkers(painter: &Painter, rect: Rect) {
    let tex_mgr = painter.ctx().tex_manager();
    let texture = painter.ctx().memory_mut(|mem| {
        const NAME: &str = "checker_background";

        let cache = mem.caches.cache::<FrameCacheDyn<TextureHandle, 1>>();
        cache.get_or_else_insert(NAME, || {
            let dark_color = Color32::from_gray(32);
            let bright_color = Color32::from_gray(128);
            let data = [dark_color, bright_color, bright_color, dark_color]
                .iter()
                .flat_map(|c| c.to_array())
                .collect::<Vec<_>>();
            let img = ColorImage::from_rgba_premultiplied([2, 2], &data);
            let tex_id =
                tex_mgr
                    .write()
                    .alloc(NAME.to_string(), img.into(), TextureOptions::NEAREST_REPEAT);
            TextureHandle::new(tex_mgr, tex_id)
        })
    });
    let mut mesh = Mesh::with_texture(texture.id());
    let rect = rect.shrink(0.5); // Small hack to avoid the checkers from peeking through the sides
    let uv = Rect::from_min_max(rect.min / CHECKER_SIZE, rect.max / CHECKER_SIZE);
    mesh.add_rect_with_uv(rect, uv, Color32::WHITE);

    painter.add(Shape::mesh(mesh));
}

fn draw_gradient(ui: &mut Ui, gradient: &Gradient, rect: Rect, use_alpha: bool) {
    const TEX_SIZE: usize = 256;

    let texture_mgr = ui.ctx().tex_manager();
    let texture_cache_id = ui.auto_id_with("gradient_texture").with(use_alpha);
    let texture = ui.memory_mut(|mem| {
        let cache = mem.caches.cache::<FrameCacheDyn<TextureHandle, 1>>();
        let mut tex = cache.get_or_else_insert(texture_cache_id, || {
            let img = ColorImage::new([TEX_SIZE, 1], Color32::BLACK);
            let tex_id = texture_mgr.write().alloc(
                "gradient".to_string(),
                img.into(),
                TextureOptions::LINEAR,
            );
            TextureHandle::new(texture_mgr, tex_id)
        });
        tex.set(
            ColorImage::from_rgba_premultiplied(
                [TEX_SIZE, 1],
                &gradient
                    .linear_eval(TEX_SIZE, !use_alpha)
                    .iter()
                    .flat_map(|c| c.to_array())
                    .collect::<Vec<_>>(),
            ),
            TextureOptions::LINEAR,
        );
        tex
    });

    // draw rect using texture
    let mut mesh = Mesh::with_texture(texture.id());
    mesh.add_rect_with_uv(
        rect,
        Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)),
        Color32::WHITE,
    );
    ui.painter().add(Shape::mesh(mesh));
}

fn gradient_box(
    ui: &mut Ui,
    gradient: &mut Gradient,
    gradient_rect: Rect,
    visuals: &WidgetVisuals,
) -> Option<usize> {
    const SOLID_HEIGHT: f32 = 8.;
    background_checkers(
        ui.painter(),
        gradient_rect.with_max_y(gradient_rect.bottom()),
    );

    let mut new_stop = None;

    let response = ui.allocate_rect(gradient_rect, Sense::click());
    if response.double_clicked_by(PointerButton::Primary) {
        let x = response.interact_pointer_pos().unwrap().x;
        let t = ((x - gradient_rect.left()) / gradient_rect.width()).clamp(0., 1.);
        let color = gradient.interpolator().sample_at(t).unwrap();
        new_stop = Some(gradient.stops.len());
        gradient.stops.push((t, color.into()));
    }

    for (y, use_alpha) in [
        gradient_rect.top(),
        gradient_rect.bottom() - SOLID_HEIGHT,
        gradient_rect.bottom(),
    ]
    .windows(2)
    .zip([true, false])
    {
        let (top, bottom) = (y[0], y[1]);
        draw_gradient(
            ui,
            gradient,
            gradient_rect.with_min_y(top).with_max_y(bottom),
            use_alpha,
        );
    }
    ui.painter()
        .rect_stroke(gradient_rect, 0.0, visuals.bg_stroke); // outline

    new_stop
}

fn control_widgets(ui: &mut Ui, gradient: &mut Gradient, selected_stop: &mut Option<usize>) {
    ui.horizontal(|ui| {
        let add_button = Button::new("➕");
        let add_button_response = ui.add(add_button).on_hover_text("Add stop");
        if add_button_response.clicked() {
            let t = if gradient.stops.len() <= 1 {
                0.5
            } else {
                let sorted_stops = gradient.argsort();
                *selected_stop = selected_stop.map(|idx| sorted_stops[idx]);
                gradient.sort();

                let insertion_idx = selected_stop.unwrap_or(gradient.stops.len() - 1).max(1);
                let right_t = gradient.stops[insertion_idx].0;
                let left_t = (insertion_idx > 0)
                    .then(|| gradient.stops[insertion_idx - 1].0)
                    .unwrap_or(0.0);
                0.5 * (left_t + right_t)
            };
            let col = gradient.interpolator().sample_at(t).unwrap();
            gradient.stops.push((t, col.into()));
            *selected_stop = Some(gradient.stops.len());
        };
        let remove_button = Button::new("➖");
        let can_remove = selected_stop.is_some() && gradient.stops.len() > 1;
        if can_remove {
            let remove_button_response = ui.add(remove_button);
            if remove_button_response.clicked() {
                gradient.stops.remove(selected_stop.unwrap());
            }
            remove_button_response
        } else {
            ui.add_enabled(false, remove_button)
        }
        .on_hover_text("Remove stop");

        ComboBox::from_id_source(ui.auto_id_with(0))
            .selected_text(gradient.interpolation_method.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut gradient.interpolation_method,
                    InterpolationMethod::Linear,
                    InterpolationMethod::Linear.to_string(),
                );
                ui.selectable_value(
                    &mut gradient.interpolation_method,
                    InterpolationMethod::Constant,
                    InterpolationMethod::Constant.to_string(),
                );
            })
            .response
            .on_hover_text("Interpolation method");
    });
}

fn gradient_stop(
    ui: &mut Ui,
    rect: Rect,
    idx: usize,
    (t, color): (&mut f32, &mut Hsva),
    selected_stop: &mut Option<usize>,
    did_interact: &mut bool,
    added_stop: &mut Option<usize>,
) {
    let is_selected = matches!(selected_stop, Some(i) if *i == idx);
    let popup_id = Id::new(ui.id()).with("popup").with(idx);
    let x = lerp(rect.left()..=rect.right(), *t);

    let tick_rect = Rect::from_center_size(
        pos2(x, rect.bottom() + TICK_SQUARE_SIZE * 0.5 - TICK_OFFSET),
        Vec2::splat(TICK_SQUARE_SIZE),
    );
    let mut tick_response = ui.allocate_rect(
        tick_rect.expand(5.).with_min_y(rect.top()),
        Sense::click_and_drag(),
    );

    let mut toggled_popup = false;

    if tick_response.dragged_by(PointerButton::Primary) {
        *t = (*t + tick_response.drag_delta().x / rect.width()).clamp(0., 1.);
        *selected_stop = Some(idx);
        ui.memory_mut(|mem| {
            if !mem.is_popup_open(popup_id) {
                // close any open pop-up unless it belongs here
                mem.close_popup();
            }
        });
        *did_interact = true;
    } else if tick_response.clicked() || added_stop.is_some_and(|added_stop| added_stop == idx) {
        ui.memory_mut(|mem| mem.open_popup(popup_id));
        *selected_stop = Some(idx);
        toggled_popup = true;
        *did_interact = true;
    }
    const COLOR_SLIDER_WIDTH: f32 = 200.;
    if ui.memory(|mem| mem.is_popup_open(popup_id)) {
        let area_response = Area::new(popup_id)
            .order(Order::Foreground)
            .fixed_pos(tick_response.rect.max)
            .constrain(true)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().slider_width = COLOR_SLIDER_WIDTH;
                Frame::popup(ui.style()).show(ui, |ui| {
                    if color_picker_hsva_2d(ui, color, Alpha::BlendOrAdditive) {
                        tick_response.mark_changed();
                        *did_interact = true;
                    }
                });
            })
            .response;

        if !toggled_popup
            && (ui.input(|i| i.key_pressed(Key::Escape)) || area_response.clicked_elsewhere())
        {
            ui.memory_mut(|mem| mem.close_popup());
            // *did_interact = true;
        }
    }

    let mut visuals = ui.style().interact_selectable(&tick_response, is_selected);
    if is_selected {
        visuals.fg_stroke.width = 3.;
        visuals.bg_stroke = ui.style().visuals.widgets.hovered.bg_stroke;
        visuals.bg_stroke.width = 3.;
    }
    let mut painter = ui.painter().clone();

    if is_selected {
        // to draw the selected stop on top of the others, create a new layer
        painter.set_layer_id(LayerId::new(
            Order::Middle,
            ui.auto_id_with("selected stop"),
        ));
    }
    painter.add(Shape::vline(
        x,
        tick_rect.top()..=rect.top(),
        Stroke::new(visuals.fg_stroke.width, Color32::WHITE),
    ));
    painter.add(Shape::dashed_line(
        &[pos2(x, tick_rect.top()), pos2(x, rect.top())],
        Stroke::new(visuals.fg_stroke.width, Color32::BLACK),
        2.,
        2. + visuals.fg_stroke.width / 2.,
    ));
    painter.rect_filled(tick_rect, 0.0, color.to_opaque());
    painter.rect_stroke(
        tick_rect.expand(visuals.fg_stroke.width / 2.),
        0.0,
        visuals.bg_stroke,
    );
    painter.rect_stroke(tick_rect, 0.0, visuals.fg_stroke);
}

/// A color gradient editor widget
pub fn gradient_editor(ui: &mut Ui, gradient: &mut Gradient) {
    let selected_stop_id = ui.auto_id_with("selected_stop");

    ui.vertical(|ui| {
        let mut selected_stop: Option<usize> =
            ui.memory_mut(|mem| mem.data.remove_temp(selected_stop_id));

        control_widgets(ui, gradient, &mut selected_stop);
        let mut added_stop: Option<usize> = None;

        let minimum_size = vec2(
            ui.spacing().slider_width,
            ui.spacing().interact_size.y * 1.7,
        );
        ui.set_min_size(minimum_size);
        let desired_size = minimum_size * vec2(4., 1.);
        let requested_size = ui.available_size().max(minimum_size).min(desired_size);

        let (rect, response) = ui.allocate_at_least(requested_size, Sense::hover());

        let mut did_interact = false;

        if ui.is_rect_visible(rect) {
            let visuals = *ui.style().noninteractive();

            let gradient_rect = rect
                .with_max_y(rect.max.y - TICK_OFFSET)
                .shrink2(vec2(TICK_SQUARE_SIZE * 0.5 + 2., 0.));

            if let Some(new_stop) = gradient_box(ui, gradient, gradient_rect, &visuals) {
                added_stop = Some(new_stop);
            }

            for (idx, (t, color)) in gradient.stops.iter_mut().enumerate() {
                gradient_stop(
                    ui,
                    gradient_rect,
                    idx,
                    (t, color),
                    &mut selected_stop,
                    &mut did_interact,
                    &mut added_stop,
                );
            }
        }
        if response.clicked_elsewhere() && !did_interact {
            selected_stop = None;
        }

        ui.memory_mut(|mem| {
            if let Some(idx) = selected_stop {
                mem.data.insert_temp(selected_stop_id, idx)
            }
        })
    });
}
