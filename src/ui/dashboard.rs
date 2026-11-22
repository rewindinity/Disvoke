use crate::app::{DisvokeApp, Page};
use crate::theme::{
    self, ACCENT, ACCENT_DIM, BG_2, BG_3, BORDER, RADIUS_LG, SUCCESS, TEXT_0, TEXT_1, TEXT_2,
    format_number,
};
use egui::{Align, Align2, CornerRadius, Frame, Layout, Margin, RichText, Stroke, StrokeKind, Ui, vec2};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui) {
    theme::page_title(ui, app.i18n.t("dashboard.title"));
    ui.add_space(16.0);

    theme::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;

            let (av_rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), egui::Sense::hover());
            let painter = ui.painter();
            painter.circle_filled(av_rect.center(), 24.0, BG_3);

            if let Some(tex) = &app.avatar_texture {
                egui::Image::new((tex.id(), vec2(48.0, 48.0))).paint_at(ui, av_rect);
            } else {
                let initial = app
                    .validated_user
                    .as_ref()
                    .and_then(|u| u.username.chars().next())
                    .map(|c| c.to_uppercase().to_string())
                    .unwrap_or_else(|| "?".to_string());
                painter.text(
                    av_rect.center(),
                    Align2::CENTER_CENTER,
                    initial,
                    egui::FontId::proportional(20.0),
                    TEXT_1,
                );
            }
            painter.circle_stroke(av_rect.center(), 24.0, Stroke::new(2.0_f32, BORDER));

            let (name, sub, badge, badge_color) = if let Some(user) = &app.validated_user {
                (
                    user.username.clone(),
                    format!("ID: {}", user.id),
                    "✓ Active".to_string(),
                    SUCCESS,
                )
            } else {
                (
                    app.i18n.t("dashboard.notLoggedIn").to_string(),
                    app.i18n.t("dashboard.configureToken").to_string(),
                    "—".to_string(),
                    TEXT_2,
                )
            };

            ui.vertical(|ui| {
                ui.add_space(4.0);
                ui.label(RichText::new(name).size(16.0).strong().color(TEXT_0));
                ui.label(RichText::new(sub).size(12.0).color(TEXT_2));
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Frame::new()
                    .fill(BG_3)
                    .stroke(Stroke::new(1.0_f32, BORDER))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.label(RichText::new(badge).size(12.0).color(badge_color));
                    });
            });
        });
    });

    ui.add_space(20.0);

    let (total_str, servers_str, dms_str) = if let Some(pack) = &app.pack_data {
        (
            format_number(pack.total_messages),
            format_number(pack.servers.len()),
            format_number(pack.dms.len()),
        )
    } else {
        ("—".to_string(), "—".to_string(), "—".to_string())
    };
    let selected_str = format_number(app.selected_ids.len());

    let cards = [
        (total_str, app.i18n.t("dashboard.statTotal"), false),
        (servers_str, app.i18n.t("dashboard.statServers"), false),
        (dms_str, app.i18n.t("dashboard.statDMs"), false),
        (selected_str, app.i18n.t("dashboard.statSelected"), true),
    ];

    let gap = 12.0;
    let total_w = ui.available_width();
    let card_w = ((total_w - gap * 3.0) / 4.0).max(120.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (value, label, accent) in cards {
            let (rect, _) = ui.allocate_exact_size(vec2(card_w, 92.0), egui::Sense::hover());
            let painter = ui.painter();
            let fill = if accent { ACCENT_DIM } else { BG_2 };
            let stroke = Stroke::new(1.0_f32, if accent { ACCENT } else { BORDER });
            painter.rect_filled(rect, CornerRadius::same(RADIUS_LG), fill);
            painter.rect_stroke(rect, CornerRadius::same(RADIUS_LG), stroke, StrokeKind::Inside);

            let num_color = if accent { ACCENT } else { TEXT_0 };
            painter.text(
                egui::pos2(rect.center().x, rect.top() + 36.0),
                Align2::CENTER_CENTER,
                &value,
                egui::FontId::proportional(28.0),
                num_color,
            );
            painter.text(
                egui::pos2(rect.center().x, rect.bottom() - 22.0),
                Align2::CENTER_CENTER,
                label.to_uppercase(),
                egui::FontId::proportional(10.5),
                TEXT_2,
            );
        }
    });

    ui.add_space(24.0);

    let load_pack_label = app.i18n.t("dashboard.loadPack").to_string();
    let settings_label = app.i18n.t("dashboard.settings").to_string();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        if theme::primary_button(ui, &load_pack_label, true).clicked() {
            app.page = Page::Pack;
        }
        if theme::outline_button(ui, &settings_label, true).clicked() {
            app.page = Page::Settings;
        }
    });
}
