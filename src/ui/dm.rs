use crate::app::{DisvokeApp, Page, StatusKind};
use crate::theme::{
    self, ACCENT, DANGER, SUCCESS, SUCCESS_DIM, TEXT_0, TEXT_1, TEXT_2, format_number,
};
use crate::store::EditContentMode;
use egui::{CornerRadius, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, ctx: &egui::Context) {
    let title = app.i18n.t("dm.title").to_string();
    let subtitle = app.i18n.t("dm.subtitle").to_string();
    let section = app.i18n.t("dm.connectionSection").to_string();
    let token_lbl = app.i18n.t("settings.tokenLabel").to_string();
    let token_ph = app.i18n.t("settings.tokenPlaceholder").to_string();
    let toggle_lbl = app
        .i18n
        .t(if app.show_token {
            "settings.hideToken"
        } else {
            "settings.showToken"
        })
        .to_string();
    let channel_lbl = app.i18n.t("dm.channelLabel").to_string();
    let channel_ph = app.i18n.t("dm.channelPlaceholder").to_string();
    let channel_hint = app.i18n.t("dm.channelHint").to_string();
    let search_lbl = app.i18n.t("dm.searchLabel").to_string();
    let search_ph = app.i18n.t("dm.searchPlaceholder").to_string();
    let fetch_lbl = app.i18n.t("dm.fetchBtn").to_string();
    let cancel_lbl = app.i18n.t("dm.cancelBtn").to_string();
    let review_lbl = app.i18n.t("dm.reviewBtn").to_string();
    let delete_lbl = app.i18n.t("dm.deleteNowBtn").to_string();

    ScrollArea::vertical()
        .id_salt("direct_dm_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            theme::page_title(ui, &title);
            ui.add_space(6.0);
            ui.label(RichText::new(subtitle).size(13.5).color(TEXT_1));
            ui.add_space(20.0);

            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::section_title(ui, &section);

                theme::field_label(ui, &token_lbl);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let input_w = (ui.available_width() - 90.0).max(200.0);
                    let resp = ui.add(
                        TextEdit::singleline(&mut app.draft_settings.token)
                            .password(!app.show_token)
                            .hint_text(&token_ph)
                            .desired_width(input_w)
                            .margin(Margin::symmetric(12, 8)),
                    );
                    if resp.changed() {
                        app.settings.token = app.draft_settings.token.trim().to_string();
                    }
                    if theme::small_button(ui, &toggle_lbl, true).clicked() {
                        app.show_token = !app.show_token;
                    }
                });

                ui.add_space(14.0);

                theme::field_label(ui, &channel_lbl);
                ui.add_space(4.0);
                ui.add(
                    TextEdit::singleline(&mut app.dm.target_input)
                        .hint_text(&channel_ph)
                        .desired_width((ui.available_width() - 8.0).max(200.0))
                        .margin(Margin::symmetric(12, 8)),
                );
                ui.add_space(4.0);
                theme::field_hint(ui, &channel_hint);

                ui.add_space(14.0);

                theme::field_label(ui, &search_lbl);
                ui.add_space(4.0);
                let search_resp = ui.add(
                    TextEdit::singleline(&mut app.dm.search_filter)
                        .hint_text(&search_ph)
                        .desired_width((ui.available_width() - 8.0).max(200.0))
                        .margin(Margin::symmetric(12, 8)),
                );
                if search_resp.changed() && app.dm.fetched_info.is_some() {
                    app.apply_dm_filter();
                }

                if app.settings.edit_before_delete.enabled {
                    ui.add_space(10.0);
                    let edit_note = match app.settings.edit_before_delete.mode {
                        EditContentMode::Custom => app.i18n.t("progress.logEditCustom").to_string(),
                        EditContentMode::Random => app.i18n.fmt(
                            "progress.logEditRandom",
                            &[(
                                "len",
                                &app.settings.edit_before_delete.random_length.to_string(),
                            )],
                        ),
                    };
                    ui.label(RichText::new(edit_note).size(12.0).color(ACCENT));
                }

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    if app.dm.fetching {
                        if theme::danger_button(ui, &cancel_lbl, true).clicked() {
                            app.cancel_dm_fetch();
                        }
                        let progress_txt = app.i18n.fmt(
                            "dm.fetching",
                            &[
                                ("scanned", &format_number(app.dm.scanned)),
                                ("found", &format_number(app.dm.found)),
                            ],
                        );
                        ui.label(RichText::new(progress_txt).size(13.0).color(TEXT_1));
                    } else {
                        let can_fetch = !app.progress.running;
                        if theme::primary_button(ui, &fetch_lbl, can_fetch).clicked() {
                            app.start_dm_fetch(ctx);
                        }
                    }
                });

                if let Some(status) = &app.dm.status {
                    ui.add_space(12.0);
                    let color = match status.kind {
                        StatusKind::Ok => SUCCESS,
                        StatusKind::Err => DANGER,
                        StatusKind::Neutral => TEXT_1,
                    };
                    ui.label(RichText::new(&status.text).size(13.0).color(color));
                }
            });

            if let Some(info) = app.dm.fetched_info.clone() {
                ui.add_space(18.0);
                let has_messages = !app.filtered_messages.is_empty();
                let selected_count = app.selected_ids.len();

                let frame = egui::Frame::new()
                    .fill(if has_messages { SUCCESS_DIM } else { theme::BG_2 })
                    .stroke(Stroke::new(
                        1.0_f32,
                        if has_messages { SUCCESS } else { theme::BORDER },
                    ))
                    .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                    .inner_margin(Margin::symmetric(24, 20));

                frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if !has_messages {
                        ui.label(
                            RichText::new(app.i18n.t("dm.noMessages"))
                                .size(13.5)
                                .color(TEXT_2),
                        );
                    } else {
                        let summary = app.i18n.fmt(
                            "dm.fetchedSummary",
                            &[
                                ("name", &info.channel_name),
                                ("id", &info.channel_id),
                                ("found", &format_number(app.filtered_messages.len())),
                                ("selected", &format_number(selected_count)),
                            ],
                        );
                        ui.label(RichText::new(summary).size(14.0).strong().color(TEXT_0));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.0;
                            if theme::outline_button(ui, &review_lbl, true).clicked() {
                                app.page = Page::Review;
                            }
                            let can_delete = selected_count > 0 && !app.progress.running;
                            if theme::danger_button(ui, &delete_lbl, can_delete).clicked() {
                                app.request_start_deletion();
                            }
                        });
                    }
                });
            }
        });
}
