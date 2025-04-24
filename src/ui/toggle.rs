use eframe::egui::{self, Ui, Response, Sense, Color32, Shape};

pub fn toggle_switch(ui: &mut Ui, on: &mut bool, id_source: &str) -> Response {
    let id = ui.make_persistent_id(id_source);
    let size = ui.spacing().interact_size.y * egui::vec2(1.5, 1.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if response.clicked() {
        *on = !*on;
        ui.memory_mut(|m| m.data.insert_temp(id, *on));
        log::debug!("Toggle switch '{}' changed to {}", id_source, *on);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let style_visuals = ui.style().visuals.clone();
        let (small_rect, big_rect) = {
            let small = rect.shrink(2.0);
            let big = small.expand(1.0);
            (small, big)
        };

        let t = ui.ctx().animate_bool_with_time(id, *on, 0.2);
        let color = {
            let start = style_visuals.widgets.inactive.bg_fill.to_array();
            let end = style_visuals.widgets.active.bg_fill.to_array();
            let r = (start[0] as f32 + t * (end[0] as f32 - start[0] as f32)) as u8;
            let g = (start[1] as f32 + t * (end[1] as f32 - start[1] as f32)) as u8;
            let b = (start[2] as f32 + t * (end[2] as f32 - start[2] as f32)) as u8;
            let a = (start[3] as f32 + t * (end[3] as f32 - start[3] as f32)) as u8;
            Color32::from_rgba_premultiplied(r, g, b, a)
        };

        ui.painter().add(Shape::rect_filled(
            big_rect,
            4.0,
            visuals.bg_fill,
        ));
        let base_pos = small_rect.min + egui::vec2(small_rect.height() / 2.0, small_rect.height() / 2.0);
        let offset = egui::vec2(small_rect.width() - small_rect.height(), 0.0);
        let interpolated_offset = egui::lerp(egui::vec2(0.0, 0.0)..=offset, t);
        let circle_pos = base_pos + interpolated_offset;
        ui.painter().add(Shape::circle_filled(
            circle_pos,
            small_rect.height() / 2.5,
            color,
        ));
    }

    response
}