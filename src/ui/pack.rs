use crate::app::DisvokeApp;
use crate::icons::{self, Icon};
use crate::theme::{self, Palette, RADIUS_LG, format_number};
use crate::ui::dm::render_active_safety_chips;
use egui::{
    Align2, Color32, Context, CornerRadius, CursorIcon, FontId, Margin, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, pos2, vec2,
};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, ctx: &Context, p: &Palette) {
    let title = app.i18n.t("pack.title").to_string();
    let subtitle = app.i18n.t("pack.subtitle").to_string();
    let guide_title = app.i18n.t("pack.guideTitle").to_string();
    let guide_s1 = app.i18n.t("pack.guideStep1").to_string();
    let guide_s2 = app.i18n.t("pack.guideStep2").to_string();
    let guide_s3 = app.i18n.t("pack.guideStep3").to_string();

    ScrollArea::vertical()
        .id_salt("pack_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            theme::centered_column(ui, 880.0, |ui| {
                theme::page_header(ui, p, Icon::Folder, &title, None);
                ui.add_space(6.0);
                theme::render_inline_rich_text(ui, p, &subtitle);
                ui.add_space(14.0);

                theme::help_drawer(
                    ui,
                    p,
                    "pack_help",
                    &guide_title,
                    &mut app.help_pack_open,
                    &[&guide_s1, &guide_s2, &guide_s3],
                );
                ui.add_space(16.0);

                let loaded = app.pack_data.is_some();
                let file_hover = ctx.input(|i| !i.raw.hovered_files.is_empty());

                let drop_width = ui.available_width();
                let drop_height = 178.0;
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
                    painter.rect_filled(drop_rect, CornerRadius::same(RADIUS_LG), p.success_dim);
                    painter.rect_stroke(
                        drop_rect,
                        CornerRadius::same(RADIUS_LG),
                        Stroke::new(2.0_f32, p.success),
                        StrokeKind::Inside,
                    );
                } else if hovered {
                    painter.rect_filled(drop_rect, CornerRadius::same(RADIUS_LG), p.accent_dim);
                    theme::paint_dashed_rect(
                        painter,
                        drop_rect.shrink(1.0),
                        Stroke::new(2.0_f32, p.accent),
                        8.0,
                        6.0,
                    );
                } else {
                    painter.rect_filled(drop_rect, CornerRadius::same(RADIUS_LG), p.bg_2);
                    theme::paint_dashed_rect(
                        painter,
                        drop_rect.shrink(1.0),
                        Stroke::new(1.5_f32, p.border),
                        8.0,
                        6.0,
                    );
                }

                let icon_color = if loaded {
                    p.success
                } else if hovered {
                    p.accent_text
                } else {
                    p.text_2
                };
                let icon_bg = if loaded {
                    p.success_dim
                } else if hovered {
                    p.accent_dim
                } else {
                    p.bg_3
                };
                let icon_box = Rect::from_center_size(
                    pos2(drop_rect.center().x, drop_rect.top() + 46.0),
                    vec2(48.0, 48.0),
                );
                painter.rect_filled(icon_box, CornerRadius::same(12), icon_bg);
                icons::paint(
                    painter,
                    icon_box.shrink(11.0),
                    if loaded { Icon::Check } else { Icon::FolderOpen },
                    icon_color,
                );

                let (status_text, status_color) = if app.pack_loading {
                    (app.i18n.t("pack.loading").to_string(), p.text_0)
                } else if let Some(err) = &app.pack_error {
                    (format!("{}: {err}", app.i18n.t("pack.errorGeneric")), p.danger)
                } else if let (Some(folder), Some(data)) = (&app.pack_folder, &app.pack_data) {
                    (
                        format!("{folder}  —  {} msg.", format_number(data.total_messages)),
                        p.text_0,
                    )
                } else {
                    (app.i18n.t("pack.clickToSelect").to_string(), p.text_2)
                };

                let status_galley = painter.layout(
                    status_text,
                    FontId::monospace(12.0),
                    status_color,
                    (drop_width - 48.0).max(100.0),
                );
                painter.galley(
                    pos2(
                        drop_rect.center().x - status_galley.size().x * 0.5,
                        drop_rect.top() + 84.0,
                    ),
                    status_galley,
                    status_color,
                );

                let btn_label = app.i18n.t("pack.selectFolder").to_string();
                let btn_galley = painter.layout_no_wrap(
                    btn_label.clone(),
                    FontId::proportional(13.0),
                    Color32::WHITE,
                );
                let btn_size = vec2(btn_galley.size().x + 54.0, btn_galley.size().y + 18.0);
                let btn_rect = Rect::from_center_size(
                    pos2(drop_rect.center().x, drop_rect.bottom() - 34.0),
                    btn_size,
                );
                let btn_fill = if app.pack_loading {
                    p.accent.gamma_multiply(0.45)
                } else if hovered {
                    p.accent_hover
                } else {
                    p.accent
                };
                painter.rect_filled(btn_rect, CornerRadius::same(theme::RADIUS), btn_fill);
                let b_ic = Rect::from_min_size(
                    pos2(btn_rect.left() + 14.0, btn_rect.center().y - 7.5),
                    vec2(15.0, 15.0),
                );
                icons::paint(painter, b_ic, Icon::Folder, Color32::WHITE);
                painter.text(
                    pos2(b_ic.right() + 7.0, btn_rect.center().y),
                    Align2::LEFT_CENTER,
                    btn_label,
                    FontId::proportional(13.0),
                    Color32::WHITE,
                );

                if !app.pack_loading
                    && drop_resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
                {
                    app.pick_pack_folder(ctx);
                }

                if let Some(pack) = &app.pack_data {
                    let total_txt = format!(
                        "{}: {}",
                        app.i18n.t("dashboard.statTotal"),
                        format_number(pack.total_messages)
                    );
                    let srv_txt = format!(
                        "{}: {}",
                        app.i18n.t("dashboard.statServers"),
                        format_number(pack.servers.len())
                    );
                    let dm_txt = format!(
                        "{}: {}",
                        app.i18n.t("dashboard.statDMs"),
                        format_number(pack.dms.len())
                    );

                    ui.add_space(12.0);
                    ui.horizontal_wrapped(|ui| {
                        theme::pill_badge(ui, &total_txt, p.accent_dim, p.accent_text, Some(p.accent_border));
                        theme::pill_badge(ui, &srv_txt, p.bg_2, p.text_1, Some(p.border));
                        theme::pill_badge(ui, &dm_txt, p.bg_2, p.text_1, Some(p.border));
                    });

                    ui.add_space(16.0);
                    let filter_title = app.i18n.t("pack.filterTitle").to_string();
                    let filter_servers_lbl = app.i18n.t("pack.filterServers").to_string();
                    let filter_dms_lbl = app.i18n.t("pack.filterDMs").to_string();
                    let filter_search_lbl = app.i18n.t("pack.filterSearch").to_string();
                    let filter_placeholder = app.i18n.t("pack.filterSearchPlaceholder").to_string();
                    let apply_label = app.i18n.t("pack.applyFilter").to_string();

                    theme::card_frame(p).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        theme::section_header(ui, p, Icon::Search, &filter_title);

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 20.0;
                            ui.checkbox(
                                &mut app.filter_servers,
                                RichText::new(filter_servers_lbl).color(p.text_0),
                            );
                            ui.checkbox(
                                &mut app.filter_dms,
                                RichText::new(filter_dms_lbl).color(p.text_0),
                            );
                        });

                        ui.add_space(12.0);
                        theme::field_label(ui, p, &filter_search_lbl);
                        ui.add_space(4.0);
                        ui.add(
                            TextEdit::singleline(&mut app.filter_search)
                                .hint_text(filter_placeholder)
                                .desired_width((ui.available_width() - 4.0).max(180.0))
                                .margin(Margin::symmetric(12, 8)),
                        );

                        ui.add_space(14.0);
                        render_active_safety_chips(app, ui, p);

                        ui.add_space(16.0);
                        if theme::primary_button(ui, p, Some(Icon::ArrowRight), &apply_label, true)
                            .clicked()
                        {
                            app.apply_filter_and_review();
                        }
                    });
                }
            });
        });
}
