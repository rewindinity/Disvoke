use crate::app::{DisvokeApp, Page, SettingsTab, StatusKind};
use crate::icons::{self, Icon};
use crate::store::EditContentMode;
use crate::theme::{self, Palette, format_number};
use egui::{CornerRadius, Margin, RichText, ScrollArea, Sense, Stroke, TextEdit, Ui, vec2};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, ctx: &egui::Context, p: &Palette) {
    let title = app.i18n.t("dm.title").to_string();
    let subtitle = app.i18n.t("dm.subtitle").to_string();
    let section = app.i18n.t("dm.connectionSection").to_string();
    let guide_title = app.i18n.t("dm.guideTitle").to_string();
    let guide_s1 = app.i18n.t("dm.guideStep1").to_string();
    let guide_s2 = app.i18n.t("dm.guideStep2").to_string();
    let guide_s3 = app.i18n.t("dm.guideStep3").to_string();

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
    let verify_lbl = app.i18n.t("settings.verifyToken").to_string();

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
            theme::centered_column(ui, 880.0, |ui| {
                theme::page_header(ui, p, Icon::Message, &title, Some(&subtitle));
                ui.add_space(16.0);

                theme::help_drawer(
                    ui,
                    p,
                    "dm_help",
                    &guide_title,
                    &mut app.help_dm_open,
                    &[&guide_s1, &guide_s2, &guide_s3],
                );
                ui.add_space(16.0);

                theme::card_frame(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    theme::section_header(ui, p, Icon::Key, &section);

                    theme::field_label(ui, p, &token_lbl);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let input_w = (ui.available_width() - 210.0).max(160.0);
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
                        let eye = if app.show_token { Icon::EyeOff } else { Icon::Eye };
                        if theme::small_button(ui, p, Some(eye), &toggle_lbl, true).clicked() {
                            app.show_token = !app.show_token;
                        }
                        if theme::small_button(ui, p, Some(Icon::Check), &verify_lbl, !app.token_checking)
                            .clicked()
                        {
                            app.verify_token(ctx);
                        }
                    });
                    if let Some(res) = &app.token_result {
                        ui.add_space(6.0);
                        let c = match res.kind {
                            StatusKind::Ok => p.success,
                            StatusKind::Err => p.danger,
                            StatusKind::Neutral => p.text_1,
                        };
                        ui.label(RichText::new(&res.text).size(12.0).color(c));
                    }

                    ui.add_space(14.0);

                    theme::field_label(ui, p, &channel_lbl);
                    ui.add_space(4.0);
                    ui.add(
                        TextEdit::singleline(&mut app.dm.target_input)
                            .hint_text(&channel_ph)
                            .desired_width((ui.available_width() - 4.0).max(180.0))
                            .margin(Margin::symmetric(12, 8)),
                    );
                    ui.add_space(4.0);
                    theme::field_hint(ui, p, &channel_hint);

                    ui.add_space(14.0);

                    theme::field_label(ui, p, &search_lbl);
                    ui.add_space(4.0);
                    let search_resp = ui.add(
                        TextEdit::singleline(&mut app.dm.search_filter)
                            .hint_text(&search_ph)
                            .desired_width((ui.available_width() - 4.0).max(180.0))
                            .margin(Margin::symmetric(12, 8)),
                    );
                    if search_resp.changed() && app.dm.fetched_info.is_some() {
                        app.apply_dm_filter();
                    }

                    ui.add_space(14.0);
                    render_active_safety_chips(app, ui, p);

                    ui.add_space(16.0);

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        if app.dm.fetching {
                            if theme::danger_button(ui, p, Some(Icon::Stop), &cancel_lbl, true).clicked() {
                                app.cancel_dm_fetch();
                            }
                            let progress_txt = app.i18n.fmt(
                                "dm.fetching",
                                &[
                                    ("scanned", &format_number(app.dm.scanned)),
                                    ("found", &format_number(app.dm.found)),
                                ],
                            );
                            ui.label(RichText::new(progress_txt).size(13.0).color(p.text_1));
                        } else {
                            let can_fetch = !app.progress.running;
                            if theme::primary_button(ui, p, Some(Icon::Search), &fetch_lbl, can_fetch)
                                .clicked()
                            {
                                app.start_dm_fetch(ctx);
                            }
                        }
                    });

                    if let Some(status) = &app.dm.status {
                        ui.add_space(12.0);
                        let color = match status.kind {
                            StatusKind::Ok => p.success,
                            StatusKind::Err => p.danger,
                            StatusKind::Neutral => p.text_1,
                        };
                        ui.label(RichText::new(&status.text).size(13.0).color(color));
                    }
                });

                if let Some(info) = app.dm.fetched_info.clone() {
                    ui.add_space(16.0);
                    let has_messages = !app.filtered_messages.is_empty();
                    let selected_count = app.selected_ids.len();

                    let frame = egui::Frame::new()
                        .fill(if has_messages { p.success_dim } else { p.bg_2 })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if has_messages { p.success } else { p.border },
                        ))
                        .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                        .inner_margin(Margin::symmetric(22, 18));

                    frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if !has_messages {
                            ui.label(
                                RichText::new(app.i18n.t("dm.noMessages"))
                                    .size(13.5)
                                    .color(p.text_2),
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
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                                icons::paint(ui.painter(), r, Icon::Check, p.success);
                                ui.label(RichText::new(summary).size(14.0).strong().color(p.text_0));
                            });
                            ui.add_space(14.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 12.0;
                                if theme::outline_button(ui, p, Some(Icon::CheckSquare), &review_lbl, true)
                                    .clicked()
                                {
                                    app.page = Page::Review;
                                }
                                let can_delete = selected_count > 0 && !app.progress.running;
                                if theme::danger_button(ui, p, Some(Icon::Trash), &delete_lbl, can_delete)
                                    .clicked()
                                {
                                    app.request_start_deletion();
                                }
                            });
                        }
                    });
                }
            });
        });
}

pub fn render_active_safety_chips(app: &mut DisvokeApp, ui: &mut Ui, p: &Palette) {
    let delay_chip = format!("{}s", app.settings.delay);
    let edit_chip = if app.settings.edit_before_delete.enabled {
        match app.settings.edit_before_delete.mode {
            EditContentMode::Custom => app.i18n.t("settings.editModeCustom").to_string(),
            EditContentMode::Random => format!(
                "{} ({})",
                app.i18n.t("settings.editModeRandom"),
                app.settings.edit_before_delete.random_length
            ),
        }
    } else {
        app.i18n.t("settings.editSection").to_string()
    };
    let batch_chip = app.i18n.t("settings.batchSection").to_string();

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
        if theme::chip_button(ui, p, Icon::History, &delay_chip, false).clicked() {
            app.open_settings_tab(SettingsTab::Cleaning);
        }
        if theme::chip_button(
            ui,
            p,
            Icon::Edit,
            &edit_chip,
            app.settings.edit_before_delete.enabled,
        )
        .clicked()
        {
            app.open_settings_tab(SettingsTab::Cleaning);
        }
        if theme::chip_button(ui, p, Icon::Shield, &batch_chip, app.settings.batch_mode.enabled)
            .clicked()
        {
            app.open_settings_tab(SettingsTab::Cleaning);
        }
    });
}
