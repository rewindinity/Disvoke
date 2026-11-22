use crate::app::{DisvokeApp, LogLevel, Page};
use crate::icons::Icon;
use crate::theme::{self, Palette, RADIUS_LG, format_number};
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Layout, Rect, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, Ui, pos2, vec2,
};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, p: &Palette) {
    let title = app.i18n.t("progress.title").to_string();
    let subtitle = app.i18n.t("progress.subtitle").to_string();

    let (status_badge, badge_bg, badge_fg) = if app.progress.in_batch_break {
        (app.i18n.t("progress.statusBreak").to_string(), p.warn_dim, p.warn)
    } else if app.progress.running {
        (app.i18n.t("progress.statusRunning").to_string(), p.accent_dim, p.accent_text)
    } else if app.progress.finished && app.progress.progress >= 1.0 {
        (app.i18n.t("progress.statusDone").to_string(), p.success_dim, p.success)
    } else if app.progress.finished {
        (app.i18n.t("progress.statusStopped").to_string(), p.warn_dim, p.warn)
    } else {
        (app.i18n.t("progress.statusReady").to_string(), p.bg_2, p.text_2)
    };

    ui.horizontal(|ui| {
        theme::page_header(ui, p, Icon::Activity, &title, Some(&subtitle));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            theme::pill_badge(ui, &status_badge, badge_bg, badge_fg, None);
        });
    });

    ui.add_space(16.0);

    let deleted = app.progress.deleted;
    let errors = app.progress.errors;
    let remaining = app.progress.total.saturating_sub(deleted + errors);

    let gap = 12.0;
    let card_w = ((ui.available_width() - gap * 2.0) / 3.0).max(110.0);
    let del_lbl = app.i18n.t("progress.deleted").to_string();
    let err_lbl = app.i18n.t("progress.errors").to_string();
    let rem_lbl = app.i18n.t("progress.remaining").to_string();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        stat_card(ui, p, card_w, &format_number(deleted), &del_lbl, p.success, p.bg_2);
        let err_bg = if errors > 0 { p.danger_dim } else { p.bg_2 };
        stat_card(ui, p, card_w, &format_number(errors), &err_lbl, p.danger, err_bg);
        stat_card(ui, p, card_w, &format_number(remaining), &rem_lbl, p.text_0, p.bg_2);
    });

    ui.add_space(16.0);

    let pct = (app.progress.progress.clamp(0.0, 1.0) * 100.0).round() as u32;
    ui.horizontal(|ui| {
        ui.label(RichText::new(&app.progress.status_text).size(12.5).color(p.text_1));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(format!("{pct}%")).size(12.5).strong().color(p.accent_text));
        });
    });
    ui.add_space(6.0);

    let bar_w = ui.available_width();
    let (bar_rect, _) = ui.allocate_exact_size(vec2(bar_w, 10.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(bar_rect, CornerRadius::same(5), p.bg_2);
    painter.rect_stroke(bar_rect, CornerRadius::same(5), Stroke::new(1.0_f32, p.border), StrokeKind::Inside);
    let fill_w = (bar_rect.width() * app.progress.progress.clamp(0.0, 1.0)).max(0.0);
    if fill_w > 0.0 {
        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_rect.height()));
        painter.rect_filled(fill_rect, CornerRadius::same(5), p.accent);
    }

    ui.add_space(14.0);

    let log_title = app.i18n.t("progress.logTitle").to_string();
    let log_empty = app.i18n.t("progress.logEmpty").to_string();
    let footer_h = 56.0;
    let log_h = (ui.available_height() - footer_h).max(180.0);

    theme::inset_frame(p).show(ui, |ui| {
        ui.set_min_size(vec2(ui.available_width(), log_h));
        ui.set_max_height(log_h);

        ui.horizontal(|ui| {
            ui.label(RichText::new(&log_title).size(12.0).strong().color(p.text_1));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format_number(app.progress.logs.len()))
                        .size(11.0)
                        .color(p.text_2),
                );
            });
        });
        ui.add_space(6.0);

        if app.progress.logs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(log_empty).size(12.5).color(p.text_2));
            });
            return;
        }

        let row_h = 19.0;
        ScrollArea::vertical()
            .id_salt("progress_logs_scroll")
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show_rows(ui, row_h, app.progress.logs.len(), |ui, range| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for idx in range {
                    let entry = &app.progress.logs[idx];
                    let color = match entry.level {
                        LogLevel::Info => p.text_1,
                        LogLevel::Success => p.success,
                        LogLevel::Error => p.danger,
                        LogLevel::Warn => p.warn,
                    };
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("[{}]", entry.time))
                                .font(FontId::monospace(11.5))
                                .color(p.text_2),
                        );
                        ui.label(
                            RichText::new(&entry.text)
                                .font(FontId::monospace(12.0))
                                .color(color),
                        );
                    });
                }
            });
    });

    ui.add_space(14.0);

    let stop_lbl = app.i18n.t("progress.stop").to_string();
    let new_session_lbl = app.i18n.t("progress.newSession").to_string();
    let can_stop = app.progress.running && !app.progress.stopping;
    let show_new_session = app.progress.finished && !app.progress.running;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        if theme::danger_button(ui, p, Some(Icon::Stop), &stop_lbl, can_stop).clicked() {
            app.stop_deletion();
        }
        if show_new_session
            && theme::outline_button(ui, p, Some(Icon::Refresh), &new_session_lbl, true).clicked()
        {
            app.page = Page::Dashboard;
        }
    });
}

fn stat_card(
    ui: &mut Ui,
    p: &Palette,
    width: f32,
    value: &str,
    label: &str,
    num_color: Color32,
    bg: Color32,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 82.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(RADIUS_LG), bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_LG),
        Stroke::new(1.0_f32, p.border),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.center().x, rect.top() + 32.0),
        Align2::CENTER_CENTER,
        value,
        FontId::proportional(28.0),
        num_color,
    );
    painter.text(
        pos2(rect.center().x, rect.bottom() - 20.0),
        Align2::CENTER_CENTER,
        label.to_uppercase(),
        FontId::proportional(10.5),
        p.text_2,
    );
}
