use crate::app::DisvokeApp;
use crate::theme::{
    self, ACCENT, ACCENT_DIM, BORDER, DANGER, RADIUS_LG, SUCCESS, SUCCESS_DIM, TEXT_0, TEXT_1,
    TEXT_2, format_number,
};
use egui::{
    Align2, Context, CornerRadius, CursorIcon, FontId, Rect, Sense, Stroke, StrokeKind, TextEdit,
    Ui, pos2, vec2,
};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, ctx: &Context) {
    egui::ScrollArea::vertical()
        .id_salt("pack_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            theme::page_title(ui, app.i18n.t("pack.title"));
            ui.add_space(6.0);
            theme::render_inline_rich_text(ui, app.i18n.t("pack.subtitle"));
            ui.add_space(20.0);

            let loaded = app.pack_data.is_some();
            let file_hover = ctx.input(|i| !i.raw.hovered_files.is_empty());

            let drop_width = ui.available_width();
            let drop_height = 180.0;
            let (drop_rect, drop_resp) = ui.allocate_exact_size(
                vec2(drop_width, drop_height),
                if app.pack_loading {
                    Sense::hover()
                } else {
                    Sense::click()
                },
            );

            let hovered = !app.pack_loading && (drop_resp.hovered() || file_hover);
            let painter = ui.painter();

            if loaded {
                painter.rect_filled(drop_rect, CornerRadius::same(RADIUS_LG), SUCCESS_DIM);
                painter.rect_stroke(
                    drop_rect,
                    CornerRadius::same(RADIUS_LG),
                    Stroke::new(2.0_f32, SUCCESS),
                    StrokeKind::Inside,
                );
            } else if hovered {
                painter.rect_filled(drop_rect, CornerRadius::same(RADIUS_LG), ACCENT_DIM);
                theme::paint_dashed_rect(
                    painter,
                    drop_rect.shrink(1.0),
                    Stroke::new(2.0_f32, ACCENT),
                    8.0,
                    6.0,
                );
            } else {
                theme::paint_dashed_rect(
                    painter,
                    drop_rect.shrink(1.0),
                    Stroke::new(2.0_f32, BORDER),
                    8.0,
                    6.0,
                );
            }

            let icon_color = if loaded {
                SUCCESS
            } else if hovered {
                ACCENT
            } else {
                TEXT_2
            };
            let icon_rect = Rect::from_center_size(
                pos2(drop_rect.center().x, drop_rect.top() + 48.0),
                vec2(48.0, 48.0),
            );
            theme::paint_folder_large(painter, icon_rect, icon_color);

            let (status_text, status_color) = if app.pack_loading {
                (app.i18n.t("pack.loading").to_string(), TEXT_0)
            } else if let Some(err) = &app.pack_error {
                (format!("{}: {err}", app.i18n.t("pack.errorGeneric")), DANGER)
            } else if let (Some(folder), Some(data)) = (&app.pack_folder, &app.pack_data) {
                (
                    format!("✓ {folder}  —  {} msg.", format_number(data.total_messages)),
                    TEXT_0,
                )
            } else {
                (app.i18n.t("pack.clickToSelect").to_string(), TEXT_2)
            };

            let status_galley = painter.layout(
                status_text,
                FontId::monospace(12.0),
                status_color,
                (drop_width - 48.0).max(100.0),
            );
            painter.galley(
                pos2(
                    drop_rect.center().x - status_galley.size().x / 2.0,
                    drop_rect.top() + 86.0,
                ),
                status_galley,
                status_color,
            );

            let btn_label = app.i18n.t("pack.selectFolder").to_string();
            let btn_galley = painter.layout_no_wrap(
                btn_label.clone(),
                FontId::proportional(13.0),
                egui::Color32::WHITE,
            );
            let btn_size = btn_galley.size() + vec2(36.0, 18.0);
            let btn_rect = Rect::from_center_size(
                pos2(drop_rect.center().x, drop_rect.bottom() - 36.0),
                btn_size,
            );
            let btn_fill = if app.pack_loading {
                ACCENT.gamma_multiply(0.45)
            } else if hovered {
                theme::ACCENT_HOVER
            } else {
                ACCENT
            };
            painter.rect_filled(btn_rect, CornerRadius::same(theme::RADIUS), btn_fill);
            painter.text(
                btn_rect.center(),
                Align2::CENTER_CENTER,
                btn_label,
                FontId::proportional(13.0),
                egui::Color32::WHITE,
            );

            if !app.pack_loading
                && drop_resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
            {
                app.pick_pack_folder(ctx);
            }

            if app.pack_data.is_some() {
                ui.add_space(20.0);
                let filter_title = app.i18n.t("pack.filterTitle").to_string();
                let filter_servers_lbl = app.i18n.t("pack.filterServers").to_string();
                let filter_dms_lbl = app.i18n.t("pack.filterDMs").to_string();
                let filter_search_lbl = app.i18n.t("pack.filterSearch").to_string();
                let filter_placeholder = app.i18n.t("pack.filterSearchPlaceholder").to_string();
                let apply_label = app.i18n.t("pack.applyFilter").to_string();

                theme::card_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    theme::section_title(ui, &filter_title);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 20.0;
                        ui.checkbox(&mut app.filter_servers, egui::RichText::new(filter_servers_lbl).color(TEXT_1));
                        ui.checkbox(&mut app.filter_dms, egui::RichText::new(filter_dms_lbl).color(TEXT_1));
                    });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(filter_search_lbl).color(TEXT_1));
                        ui.add(
                            TextEdit::singleline(&mut app.filter_search)
                                .hint_text(filter_placeholder)
                                .desired_width((ui.available_width() - 8.0).max(200.0))
                                .margin(egui::Margin::symmetric(12, 8)),
                        );
                    });

                    ui.add_space(16.0);
                    if theme::primary_button(ui, &apply_label, true).clicked() {
                        app.apply_filter_and_review();
                    }
                });
            }
        });
}
