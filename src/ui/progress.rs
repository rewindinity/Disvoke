use crate::app::{DisvokeApp, LogLevel, Page};
use crate::theme::{self, ACCENT, BG_2, DANGER, SUCCESS, TEXT_0, TEXT_2, WARN, format_number};
use egui::{CornerRadius, FontId, Rect, RichText, ScrollArea, Ui, vec2};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui) {
    theme::page_title(ui, app.i18n.t("progress.title"));
    ui.add_space(16.0);

    let deleted = app.progress.deleted;
    let errors = app.progress.errors;
    let remaining = app.progress.total.saturating_sub(deleted + errors);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 36.0;
        stat_block(ui, &format_number(deleted), app.i18n.t("progress.deleted"), TEXT_0);
        stat_block(ui, &format_number(errors), app.i18n.t("progress.errors"), DANGER);
        stat_block(ui, &format_number(remaining), app.i18n.t("progress.remaining"), TEXT_0);
    });

    ui.add_space(18.0);

    let bar_w = ui.available_width();
    let (bar_rect, _) = ui.allocate_exact_size(vec2(bar_w, 8.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(bar_rect, CornerRadius::same(4), BG_2);
    let fill_w = (bar_rect.width() * app.progress.progress.clamp(0.0, 1.0)).max(0.0);
    if fill_w > 0.0 {
        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_rect.height()));
        painter.rect_filled(fill_rect, CornerRadius::same(4), ACCENT);
    }

    ui.add_space(8.0);
    ui.label(RichText::new(&app.progress.status_text).size(12.0).color(TEXT_2));
    ui.add_space(14.0);

    let footer_h = 56.0;
    let log_h = (ui.available_height() - footer_h).max(200.0);

    theme::panel_box_frame().show(ui, |ui| {
        ui.set_min_size(vec2(ui.available_width(), log_h));
        ui.set_max_height(log_h);

        let row_h = 18.0;
        ScrollArea::vertical()
            .id_salt("progress_logs_scroll")
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show_rows(ui, row_h, app.progress.logs.len(), |ui, range| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for idx in range {
                    let entry = &app.progress.logs[idx];
                    let color = match entry.level {
                        LogLevel::Info => TEXT_2,
                        LogLevel::Success => SUCCESS,
                        LogLevel::Error => DANGER,
                        LogLevel::Warn => WARN,
                    };
                    ui.label(
                        RichText::new(format!("[{}] {}", entry.time, entry.text))
                            .font(FontId::monospace(12.0))
                            .color(color),
                    );
                }
            });
    });

    ui.add_space(16.0);

    let stop_lbl = app.i18n.t("progress.stop").to_string();
    let new_session_lbl = app.i18n.t("progress.newSession").to_string();
    let can_stop = app.progress.running && !app.progress.stopping;
    let show_new_session = app.progress.finished && !app.progress.running;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        if theme::danger_button(ui, &stop_lbl, can_stop).clicked() {
            app.stop_deletion();
        }
        if show_new_session && theme::outline_button(ui, &new_session_lbl, true).clicked() {
            app.page = Page::Dashboard;
        }
    });
}

fn stat_block(ui: &mut Ui, value: &str, label: &str, num_color: egui::Color32) {
    ui.vertical(|ui| {
        ui.label(RichText::new(value).size(34.0).strong().color(num_color));
        ui.label(
            RichText::new(label.to_uppercase())
                .size(11.0)
                .color(TEXT_2),
        );
    });
}
