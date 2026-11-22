use crate::app::{DisvokeApp, StatusKind, StatusMessage};
use crate::theme::{
    self, BG_3, BORDER, DANGER, RADIUS, SUCCESS, TEXT_0, TEXT_1, TEXT_2, format_number,
};
use egui::{ComboBox, Context, CornerRadius, DragValue, Frame, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, ctx: &Context) {
    let title = app.i18n.t("settings.title").to_string();
    let account_sec = app.i18n.t("settings.accountSection").to_string();
    let token_lbl = app.i18n.t("settings.tokenLabel").to_string();
    let token_ph = app.i18n.t("settings.tokenPlaceholder").to_string();
    let token_hint = app.i18n.t("settings.tokenHint").to_string();
    let toggle_lbl = app
        .i18n
        .t(if app.show_token {
            "settings.hideToken"
        } else {
            "settings.showToken"
        })
        .to_string();
    let verify_lbl = app.i18n.t("settings.verifyToken").to_string();

    let deletion_sec = app.i18n.t("settings.deletionSection").to_string();
    let delay_lbl = app.i18n.t("settings.delayLabel").to_string();
    let delay_hint = app.i18n.t("settings.delayHint").to_string();

    let batch_sec = app.i18n.t("settings.batchSection").to_string();
    let batch_enabled_lbl = app.i18n.t("settings.batchEnabledLabel").to_string();
    let batch_enabled_hint = app.i18n.t("settings.batchEnabledHint").to_string();
    let batch_min_lbl = app.i18n.t("settings.batchMinLabel").to_string();
    let batch_max_lbl = app.i18n.t("settings.batchMaxLabel").to_string();
    let break_min_lbl = app.i18n.t("settings.breakMinLabel").to_string();
    let break_max_lbl = app.i18n.t("settings.breakMaxLabel").to_string();
    let batch_hint = app.i18n.t("settings.batchHint").to_string();

    let history_sec = app.i18n.t("settings.historySection").to_string();
    let history_hint = app.i18n.t("settings.historyHint").to_string();
    let history_path_lbl = app.i18n.t("settings.historyPathLabel").to_string();
    let history_pick_lbl = app.i18n.t("settings.historyPickPath").to_string();
    let history_load_lbl = app.i18n.t("settings.historyLoad").to_string();
    let history_clear_lbl = app.i18n.t("settings.historyClear").to_string();

    let lang_sec = app.i18n.t("settings.languageSection").to_string();
    let lang_lbl = app.i18n.t("settings.languageLabel").to_string();
    let save_lbl = app.i18n.t("settings.saveBtn").to_string();

    let locales = app.i18n.list_locales();
    let selected_lang_name = app.i18n.display_name(&app.draft_settings.lang).to_string();

    ScrollArea::vertical()
        .id_salt("settings_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            theme::page_title(ui, &title);
            ui.add_space(16.0);

            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::section_title(ui, &account_sec);
                theme::field_label(ui, &token_lbl);
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let input_w = (ui.available_width() - 90.0).max(200.0);
                    ui.add(
                        TextEdit::singleline(&mut app.draft_settings.token)
                            .password(!app.show_token)
                            .hint_text(&token_ph)
                            .desired_width(input_w)
                            .margin(Margin::symmetric(12, 8)),
                    );
                    if theme::small_button(ui, &toggle_lbl, true).clicked() {
                        app.show_token = !app.show_token;
                    }
                });

                ui.add_space(4.0);
                theme::field_hint(ui, &token_hint);
                ui.add_space(14.0);

                if theme::primary_button(ui, &verify_lbl, !app.token_checking).clicked() {
                    app.verify_token(ctx);
                }

                if let Some(res) = &app.token_result {
                    ui.add_space(10.0);
                    render_status(ui, res);
                }
            });

            ui.add_space(16.0);

            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::section_title(ui, &deletion_sec);
                theme::field_label(ui, &delay_lbl);
                ui.add_space(4.0);

                ui.add_sized(
                    [100.0, 32.0],
                    DragValue::new(&mut app.draft_settings.delay)
                        .range(1..=30)
                        .speed(0.1),
                );
                ui.add_space(4.0);
                theme::field_hint(ui, &delay_hint);
            });

            ui.add_space(16.0);

            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::section_title(ui, &batch_sec);
                ui.checkbox(
                    &mut app.draft_settings.batch_mode.enabled,
                    RichText::new(&batch_enabled_lbl).strong().color(TEXT_0),
                );
                ui.add_space(4.0);
                theme::field_hint(ui, &batch_enabled_hint);

                if app.draft_settings.batch_mode.enabled {
                    ui.add_space(14.0);
                    let col_w = ((ui.available_width() - 16.0) / 2.0).max(160.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;
                        ui.vertical(|ui| {
                            ui.set_width(col_w);
                            theme::field_label(ui, &batch_min_lbl);
                            ui.add_space(4.0);
                            ui.add_sized(
                                [col_w, 32.0],
                                DragValue::new(&mut app.draft_settings.batch_mode.min_batch)
                                    .range(1..=9999),
                            );
                        });
                        ui.vertical(|ui| {
                            ui.set_width(col_w);
                            theme::field_label(ui, &batch_max_lbl);
                            ui.add_space(4.0);
                            ui.add_sized(
                                [col_w, 32.0],
                                DragValue::new(&mut app.draft_settings.batch_mode.max_batch)
                                    .range(1..=9999),
                            );
                        });
                    });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;
                        ui.vertical(|ui| {
                            ui.set_width(col_w);
                            theme::field_label(ui, &break_min_lbl);
                            ui.add_space(4.0);
                            ui.add_sized(
                                [col_w, 32.0],
                                DragValue::new(&mut app.draft_settings.batch_mode.min_break_min)
                                    .range(1..=9999),
                            );
                        });
                        ui.vertical(|ui| {
                            ui.set_width(col_w);
                            theme::field_label(ui, &break_max_lbl);
                            ui.add_space(4.0);
                            ui.add_sized(
                                [col_w, 32.0],
                                DragValue::new(&mut app.draft_settings.batch_mode.max_break_min)
                                    .range(1..=9999),
                            );
                        });
                    });

                    ui.add_space(10.0);
                    theme::field_hint(ui, &batch_hint);
                }
            });

            ui.add_space(16.0);

            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::section_title(ui, &history_sec);
                theme::field_hint(ui, &history_hint);
                ui.add_space(12.0);

                theme::field_label(ui, &history_path_lbl);
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let path_w = (ui.available_width() - 105.0).max(200.0);
                    Frame::new()
                        .fill(BG_3)
                        .stroke(Stroke::new(1.0_f32, BORDER))
                        .corner_radius(CornerRadius::same(RADIUS))
                        .inner_margin(Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.set_width(path_w - 24.0);
                            let (txt, color) = if app.draft_settings.history_path.is_empty() {
                                ("No file selected — history stored in memory only", TEXT_2)
                            } else {
                                (app.draft_settings.history_path.as_str(), TEXT_1)
                            };
                            ui.add(egui::Label::new(RichText::new(txt).size(12.5).color(color)).truncate());
                        });

                    if theme::small_button(ui, &history_pick_lbl, true).clicked() {
                        app.pick_history_save_path(ctx);
                    }
                });

                ui.add_space(6.0);
                theme::field_hint(
                    ui,
                    &format!("{} IDs in memory", format_number(app.history.deleted_ids.len())),
                );

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if theme::small_button(ui, &history_load_lbl, true).clicked() {
                        app.pick_history_open_file(ctx);
                    }
                    if theme::small_button(ui, &history_clear_lbl, true).clicked() {
                        app.clear_history();
                    }
                });

                if let Some(res) = &app.history_result {
                    ui.add_space(10.0);
                    render_status(ui, res);
                }
            });

            ui.add_space(16.0);

            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                theme::section_title(ui, &lang_sec);
                theme::field_label(ui, &lang_lbl);
                ui.add_space(4.0);

                ComboBox::from_id_salt("settings_lang_combo")
                    .width(200.0)
                    .selected_text(&selected_lang_name)
                    .show_ui(ui, |ui| {
                        for loc in &locales {
                            ui.selectable_value(
                                &mut app.draft_settings.lang,
                                loc.code.clone(),
                                &loc.name,
                            );
                        }
                    });
            });

            ui.add_space(18.0);

            if theme::primary_button(ui, &save_lbl, true).clicked() {
                app.save_settings();
            }

            if let Some((res, _)) = &app.save_result {
                ui.add_space(10.0);
                render_status(ui, res);
            }

            ui.add_space(16.0);
        });
}

fn render_status(ui: &mut Ui, status: &StatusMessage) {
    let color = match status.kind {
        StatusKind::Ok => SUCCESS,
        StatusKind::Err => DANGER,
        StatusKind::Neutral => TEXT_1,
    };
    ui.label(RichText::new(&status.text).size(13.0).color(color));
}
