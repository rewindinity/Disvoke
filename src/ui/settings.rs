use crate::app::{DisvokeApp, SettingsTab, StatusKind, StatusMessage};
use crate::icons::{self, Icon};
use crate::store::{AccentColor, EditContentMode, ThemeMode};
use crate::theme::{self, Palette, RADIUS, RADIUS_LG, format_number};
use egui::{
    Align2, Color32, ComboBox, Context, CornerRadius, CursorIcon, DragValue, FontId, Frame,
    Margin, Rect, RichText, ScrollArea, Sense, Slider, Stroke, StrokeKind, TextEdit, Ui, pos2,
    vec2,
};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, ctx: &Context, p: &Palette) {
    let title = app.i18n.t("settings.title").to_string();
    let subtitle = app.i18n.t("settings.subtitle").to_string();

    let tab_account = app.i18n.t("settings.tabAccount").to_string();
    let tab_cleaning = app.i18n.t("settings.tabCleaning").to_string();
    let tab_history = app.i18n.t("settings.tabHistory").to_string();
    let tab_appearance = app.i18n.t("settings.tabAppearance").to_string();
    let save_lbl = app.i18n.t("settings.saveBtn").to_string();
    let unsaved_lbl = app.i18n.t("settings.unsavedHint").to_string();

    ScrollArea::vertical()
        .id_salt("settings_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            theme::centered_column(ui, 880.0, |ui| {
                theme::page_header(ui, p, Icon::Settings, &title, Some(&subtitle));
                ui.add_space(16.0);

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                    let tabs = [
                        (SettingsTab::Account, Icon::Key, &tab_account),
                        (SettingsTab::Cleaning, Icon::Shield, &tab_cleaning),
                        (SettingsTab::History, Icon::History, &tab_history),
                        (SettingsTab::Appearance, Icon::Palette, &tab_appearance),
                    ];
                    for (tab, icon, label) in tabs {
                        let active = app.settings_tab == tab;
                        if theme::chip_button(ui, p, icon, label, active).clicked() {
                            app.settings_tab = tab;
                        }
                    }
                });

                ui.add_space(16.0);

                match app.settings_tab {
                    SettingsTab::Account => render_account_tab(app, ui, ctx, p),
                    SettingsTab::Cleaning => render_cleaning_tab(app, ui, p),
                    SettingsTab::History => render_history_tab(app, ui, ctx, p),
                    SettingsTab::Appearance => render_appearance_tab(app, ui, ctx, p),
                }

                ui.add_space(18.0);

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    if theme::primary_button(ui, p, Some(Icon::Save), &save_lbl, true).clicked() {
                        app.save_settings(ctx);
                    }
                    if let Some((res, _)) = &app.save_result {
                        render_status(ui, p, res);
                    } else if app.has_unsaved_settings() {
                        theme::pill_badge(ui, &unsaved_lbl, p.warn_dim, p.warn, None);
                    }
                });

                ui.add_space(16.0);
            });
        });
}

fn render_account_tab(app: &mut DisvokeApp, ui: &mut Ui, ctx: &Context, p: &Palette) {
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
    let guide_title = app.i18n.t("settings.tokenGuideTitle").to_string();
    let g1 = app.i18n.t("settings.tokenGuideStep1").to_string();
    let g2 = app.i18n.t("settings.tokenGuideStep2").to_string();
    let g3 = app.i18n.t("settings.tokenGuideStep3").to_string();

    theme::card_frame(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::section_header(ui, p, Icon::Key, &account_sec);

        theme::field_label(ui, p, &token_lbl);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let input_w = (ui.available_width() - 96.0).max(180.0);
            ui.add(
                TextEdit::singleline(&mut app.draft_settings.token)
                    .password(!app.show_token)
                    .hint_text(&token_ph)
                    .desired_width(input_w)
                    .margin(Margin::symmetric(12, 8)),
            );
            let eye = if app.show_token { Icon::EyeOff } else { Icon::Eye };
            if theme::small_button(ui, p, Some(eye), &toggle_lbl, true).clicked() {
                app.show_token = !app.show_token;
            }
        });

        ui.add_space(4.0);
        theme::field_hint(ui, p, &token_hint);
        ui.add_space(14.0);

        if theme::primary_button(ui, p, Some(Icon::Check), &verify_lbl, !app.token_checking).clicked() {
            app.verify_token(ctx);
        }

        if let Some(res) = &app.token_result {
            ui.add_space(10.0);
            render_status(ui, p, res);
        }
    });

    ui.add_space(14.0);
    theme::help_drawer(
        ui,
        p,
        "token_guide",
        &guide_title,
        &mut app.help_token_open,
        &[&g1, &g2, &g3],
    );
}

fn render_cleaning_tab(app: &mut DisvokeApp, ui: &mut Ui, p: &Palette) {
    let deletion_sec = app.i18n.t("settings.deletionSection").to_string();
    let delay_lbl = app.i18n.t("settings.delayLabel").to_string();
    let delay_hint = app.i18n.t("settings.delayHint").to_string();
    let preset_fast = app.i18n.t("settings.presetFast").to_string();
    let preset_balanced = app.i18n.t("settings.presetBalanced").to_string();
    let preset_safe = app.i18n.t("settings.presetSafe").to_string();

    let edit_sec = app.i18n.t("settings.editSection").to_string();
    let edit_enabled_lbl = app.i18n.t("settings.editEnabledLabel").to_string();
    let edit_enabled_hint = app.i18n.t("settings.editEnabledHint").to_string();
    let edit_mode_lbl = app.i18n.t("settings.editModeLabel").to_string();
    let edit_mode_custom = app.i18n.t("settings.editModeCustom").to_string();
    let edit_mode_random = app.i18n.t("settings.editModeRandom").to_string();
    let edit_custom_lbl = app.i18n.t("settings.editCustomLabel").to_string();
    let edit_custom_ph = app.i18n.t("settings.editCustomPlaceholder").to_string();
    let edit_custom_hint = app.i18n.t("settings.editCustomHint").to_string();
    let edit_random_len_lbl = app.i18n.t("settings.editRandomLenLabel").to_string();
    let edit_random_len_hint = app.i18n.t("settings.editRandomLenHint").to_string();
    let edit_random_preview = app.i18n.t("settings.editRandomPreview").to_string();

    let batch_sec = app.i18n.t("settings.batchSection").to_string();
    let batch_enabled_lbl = app.i18n.t("settings.batchEnabledLabel").to_string();
    let batch_enabled_hint = app.i18n.t("settings.batchEnabledHint").to_string();
    let batch_min_lbl = app.i18n.t("settings.batchMinLabel").to_string();
    let batch_max_lbl = app.i18n.t("settings.batchMaxLabel").to_string();
    let break_min_lbl = app.i18n.t("settings.breakMinLabel").to_string();
    let break_max_lbl = app.i18n.t("settings.breakMaxLabel").to_string();
    let batch_hint = app.i18n.t("settings.batchHint").to_string();

    theme::card_frame(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::section_header(ui, p, Icon::History, &deletion_sec);
        theme::field_label(ui, p, &delay_lbl);
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for (secs, lbl) in [(2, &preset_fast), (4, &preset_balanced), (8, &preset_safe)] {
                let active = app.draft_settings.delay == secs;
                if theme::chip_button(ui, p, Icon::History, lbl, active).clicked() {
                    app.draft_settings.delay = secs;
                }
            }
        });
        ui.add_space(10.0);
        ui.add(Slider::new(&mut app.draft_settings.delay, 1..=30).suffix(" s"));
        ui.add_space(4.0);
        theme::field_hint(ui, p, &delay_hint);
    });

    ui.add_space(14.0);

    theme::card_frame(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::section_header(ui, p, Icon::Edit, &edit_sec);
        ui.checkbox(
            &mut app.draft_settings.edit_before_delete.enabled,
            RichText::new(&edit_enabled_lbl).strong().color(p.text_0),
        );
        ui.add_space(4.0);
        theme::field_hint(ui, p, &edit_enabled_hint);

        if app.draft_settings.edit_before_delete.enabled {
            ui.add_space(14.0);
            theme::field_label(ui, p, &edit_mode_lbl);
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let is_custom = app.draft_settings.edit_before_delete.mode == EditContentMode::Custom;
                if theme::chip_button(ui, p, Icon::Edit, &edit_mode_custom, is_custom).clicked() {
                    app.draft_settings.edit_before_delete.mode = EditContentMode::Custom;
                }
                if theme::chip_button(ui, p, Icon::Refresh, &edit_mode_random, !is_custom).clicked() {
                    app.draft_settings.edit_before_delete.mode = EditContentMode::Random;
                }
            });

            ui.add_space(12.0);
            match app.draft_settings.edit_before_delete.mode {
                EditContentMode::Custom => {
                    theme::field_label(ui, p, &edit_custom_lbl);
                    ui.add_space(4.0);
                    ui.add(
                        TextEdit::singleline(&mut app.draft_settings.edit_before_delete.custom_text)
                            .hint_text(&edit_custom_ph)
                            .char_limit(2000)
                            .desired_width((ui.available_width() - 4.0).max(180.0))
                            .margin(Margin::symmetric(12, 8)),
                    );
                    ui.add_space(4.0);
                    theme::field_hint(ui, p, &edit_custom_hint);
                }
                EditContentMode::Random => {
                    theme::field_label(ui, p, &edit_random_len_lbl);
                    ui.add_space(6.0);
                    ui.add(Slider::new(
                        &mut app.draft_settings.edit_before_delete.random_length,
                        1..=2000,
                    ));
                    ui.add_space(4.0);
                    theme::field_hint(ui, p, &edit_random_len_hint);

                    let sample_len = (app.draft_settings.edit_before_delete.random_length as usize).min(48);
                    let pattern = "a9Kx4mP2qR7vT1wZ8cN5bH3jL6yF0sD4gM9pW2kQ7xV1nB5t";
                    let mut preview: String = pattern.chars().cycle().take(sample_len).collect();
                    if app.draft_settings.edit_before_delete.random_length > 48 {
                        preview.push_str("...");
                    }
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&edit_random_preview).size(11.5).color(p.text_2));
                        ui.label(
                            RichText::new(preview)
                                .font(FontId::monospace(11.5))
                                .color(p.accent_text),
                        );
                    });
                }
            }
        }
    });

    ui.add_space(14.0);

    theme::card_frame(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::section_header(ui, p, Icon::Shield, &batch_sec);
        ui.checkbox(
            &mut app.draft_settings.batch_mode.enabled,
            RichText::new(&batch_enabled_lbl).strong().color(p.text_0),
        );
        ui.add_space(4.0);
        theme::field_hint(ui, p, &batch_enabled_hint);

        if app.draft_settings.batch_mode.enabled {
            ui.add_space(14.0);
            let avail = ui.available_width();
            let stacked = avail < 460.0;
            let col_w = if stacked { avail } else { ((avail - 16.0) * 0.5).max(140.0) };

            let render_pair = |ui: &mut Ui,
                               l1: &str,
                               v1: &mut u32,
                               l2: &str,
                               v2: &mut u32| {
                if stacked {
                    theme::field_label(ui, p, l1);
                    ui.add_space(4.0);
                    ui.add_sized([col_w, 30.0], DragValue::new(v1).range(1..=9999));
                    ui.add_space(10.0);
                    theme::field_label(ui, p, l2);
                    ui.add_space(4.0);
                    ui.add_sized([col_w, 30.0], DragValue::new(v2).range(1..=9999));
                } else {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;
                        ui.vertical(|ui| {
                            ui.set_width(col_w);
                            theme::field_label(ui, p, l1);
                            ui.add_space(4.0);
                            ui.add_sized([col_w, 30.0], DragValue::new(v1).range(1..=9999));
                        });
                        ui.vertical(|ui| {
                            ui.set_width(col_w);
                            theme::field_label(ui, p, l2);
                            ui.add_space(4.0);
                            ui.add_sized([col_w, 30.0], DragValue::new(v2).range(1..=9999));
                        });
                    });
                }
            };

            render_pair(
                ui,
                &batch_min_lbl,
                &mut app.draft_settings.batch_mode.min_batch,
                &batch_max_lbl,
                &mut app.draft_settings.batch_mode.max_batch,
            );
            ui.add_space(12.0);
            render_pair(
                ui,
                &break_min_lbl,
                &mut app.draft_settings.batch_mode.min_break_min,
                &break_max_lbl,
                &mut app.draft_settings.batch_mode.max_break_min,
            );
            ui.add_space(10.0);
            theme::field_hint(ui, p, &batch_hint);
        }
    });
}

fn render_history_tab(app: &mut DisvokeApp, ui: &mut Ui, ctx: &Context, p: &Palette) {
    let history_sec = app.i18n.t("settings.historySection").to_string();
    let history_hint = app.i18n.t("settings.historyHint").to_string();
    let history_path_lbl = app.i18n.t("settings.historyPathLabel").to_string();
    let history_no_path = app.i18n.t("settings.historyNoPath").to_string();
    let history_pick_lbl = app.i18n.t("settings.historyPickPath").to_string();
    let history_load_lbl = app.i18n.t("settings.historyLoad").to_string();
    let history_clear_lbl = app.i18n.t("settings.historyClear").to_string();
    let memory_badge = app.i18n.fmt(
        "settings.historyInMemory",
        &[("count", &format_number(app.history.deleted_ids.len()))],
    );

    theme::card_frame(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::section_header(ui, p, Icon::History, &history_sec);
        theme::field_hint(ui, p, &history_hint);
        ui.add_space(14.0);

        theme::field_label(ui, p, &history_path_lbl);
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let path_w = (ui.available_width() - 125.0).max(160.0);
            Frame::new()
                .fill(p.bg_3)
                .stroke(Stroke::new(1.0_f32, p.border))
                .corner_radius(CornerRadius::same(RADIUS))
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_width((path_w - 24.0).max(120.0));
                    let (txt, color) = if app.draft_settings.history_path.is_empty() {
                        (history_no_path.as_str(), p.text_2)
                    } else {
                        (app.draft_settings.history_path.as_str(), p.text_1)
                    };
                    ui.add(egui::Label::new(RichText::new(txt).size(12.5).color(color)).truncate());
                });

            if theme::small_button(ui, p, Some(Icon::Folder), &history_pick_lbl, true).clicked() {
                app.pick_history_save_path(ctx);
            }
        });

        ui.add_space(10.0);
        theme::pill_badge(ui, &memory_badge, p.accent_dim, p.accent_text, Some(p.accent_border));

        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            if theme::small_button(ui, p, Some(Icon::FolderOpen), &history_load_lbl, true).clicked() {
                app.pick_history_open_file(ctx);
            }
            if theme::small_button(
                ui,
                p,
                Some(Icon::Trash),
                &history_clear_lbl,
                !app.history.deleted_ids.is_empty(),
            )
            .clicked()
            {
                app.clear_history();
            }
        });

        if let Some(res) = &app.history_result {
            ui.add_space(10.0);
            render_status(ui, p, res);
        }
    });
}

fn render_appearance_tab(app: &mut DisvokeApp, ui: &mut Ui, ctx: &Context, p: &Palette) {
    let app_sec = app.i18n.t("settings.appearanceSection").to_string();
    let theme_lbl = app.i18n.t("settings.themeLabel").to_string();
    let dark_lbl = app.i18n.t("settings.themeDark").to_string();
    let dark_desc = app.i18n.t("settings.themeDarkDesc").to_string();
    let oled_lbl = app.i18n.t("settings.themeDarkOled").to_string();
    let oled_desc = app.i18n.t("settings.themeDarkOledDesc").to_string();
    let light_lbl = app.i18n.t("settings.themeLight").to_string();
    let light_desc = app.i18n.t("settings.themeLightDesc").to_string();
    let accent_lbl = app.i18n.t("settings.accentLabel").to_string();

    let lang_sec = app.i18n.t("settings.languageSection").to_string();
    let lang_lbl = app.i18n.t("settings.languageLabel").to_string();
    let locales = app.i18n.list_locales();
    let current_lang = app.settings.lang.clone();
    let selected_lang_name = app.i18n.display_name(&current_lang).to_string();

    theme::card_frame(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::section_header(ui, p, Icon::Palette, &app_sec);

        theme::field_label(ui, p, &theme_lbl);
        ui.add_space(8.0);

        let avail = ui.available_width();
        let stacked = avail < 620.0;
        let gap = 10.0;
        let mode_w = if stacked { avail } else { ((avail - gap * 2.0) / 3.0).max(160.0) };
        let mut chosen_theme: Option<ThemeMode> = None;

        let draw_theme_card = |ui: &mut Ui, mode: ThemeMode, icon: Icon, name: &str, desc: &str| -> bool {
            let selected = app.settings.theme == mode;
            let (rect, resp) = ui.allocate_exact_size(vec2(mode_w, 72.0), Sense::click());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                let bg = if selected {
                    p.accent_dim
                } else if resp.hovered() {
                    p.bg_hover
                } else {
                    p.bg_3
                };
                let border = if selected { p.accent } else { p.border };
                painter.rect_filled(rect, CornerRadius::same(RADIUS_LG), bg);
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(RADIUS_LG),
                    Stroke::new(if selected { 1.8_f32 } else { 1.0_f32 }, border),
                    StrokeKind::Inside,
                );

                let preview = Rect::from_min_size(pos2(rect.left() + 12.0, rect.center().y - 18.0), vec2(40.0, 36.0));
                let (pv_bg, pv_bar) = match mode {
                    ThemeMode::Dark => (Color32::from_rgb(0x13, 0x16, 0x1c), Color32::from_rgb(0x23, 0x28, 0x34)),
                    ThemeMode::DarkOled => (Color32::from_rgb(0x00, 0x00, 0x00), Color32::from_rgb(0x0c, 0x0e, 0x13)),
                    ThemeMode::Light => (Color32::from_rgb(0xf4, 0xf6, 0xf9), Color32::from_rgb(0xff, 0xff, 0xff)),
                };
                painter.rect_filled(preview, CornerRadius::same(6), pv_bg);
                painter.rect_filled(
                    Rect::from_min_size(preview.min, vec2(11.0, preview.height())),
                    CornerRadius::same(4),
                    pv_bar,
                );
                painter.rect_filled(
                    Rect::from_min_size(pos2(preview.left() + 15.0, preview.top() + 8.0), vec2(18.0, 5.0)),
                    CornerRadius::same(2),
                    p.accent,
                );
                painter.rect_stroke(preview, CornerRadius::same(6), Stroke::new(1.0_f32, p.border), StrokeKind::Inside);

                let text_x = preview.right() + 12.0;
                let right_limit = if selected { rect.right() - 28.0 } else { rect.right() - 10.0 };
                let clip = Rect::from_min_max(pos2(text_x, rect.top() + 4.0), pos2(right_limit, rect.bottom() - 4.0));
                let cp = painter.with_clip_rect(clip);
                let ic = Rect::from_min_size(pos2(text_x, rect.top() + 16.0), vec2(14.0, 14.0));
                icons::paint(&cp, ic, icon, if selected { p.accent_text } else { p.text_0 });
                cp.text(
                    pos2(ic.right() + 6.0, ic.center().y),
                    Align2::LEFT_CENTER,
                    name,
                    FontId::proportional(13.0),
                    p.text_0,
                );
                let desc_g = cp.layout(
                    desc.to_string(),
                    FontId::proportional(11.0),
                    p.text_2,
                    (right_limit - text_x).max(60.0),
                );
                cp.galley(pos2(text_x, rect.top() + 36.0), desc_g, p.text_2);
                if selected {
                    let chk = Rect::from_center_size(pos2(rect.right() - 18.0, rect.center().y), vec2(15.0, 15.0));
                    icons::paint(painter, chk, Icon::Check, p.accent_text);
                }
            }
            resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
        };

        let cards = [
            (ThemeMode::Dark, Icon::Moon, &dark_lbl, &dark_desc),
            (ThemeMode::DarkOled, Icon::Moon, &oled_lbl, &oled_desc),
            (ThemeMode::Light, Icon::Sun, &light_lbl, &light_desc),
        ];

        if stacked {
            for (mode, icon, lbl, desc) in cards {
                if draw_theme_card(ui, mode, icon, lbl, desc) {
                    chosen_theme = Some(mode);
                }
                ui.add_space(8.0);
            }
        } else {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (mode, icon, lbl, desc) in cards {
                    if draw_theme_card(ui, mode, icon, lbl, desc) {
                        chosen_theme = Some(mode);
                    }
                }
            });
        }
        if let Some(m) = chosen_theme {
            app.set_theme(ctx, m);
        }

        ui.add_space(18.0);
        theme::field_label(ui, p, &accent_lbl);
        ui.add_space(8.0);

        let accents = [
            (AccentColor::Orange, app.i18n.t("settings.accentOrange").to_string()),
            (AccentColor::Blue, app.i18n.t("settings.accentBlue").to_string()),
            (AccentColor::Green, app.i18n.t("settings.accentGreen").to_string()),
            (AccentColor::Red, app.i18n.t("settings.accentRed").to_string()),
        ];

        let two_col = avail < 520.0;
        let cols = if two_col { 2 } else { 4 };
        let ac_w = ((avail - 10.0 * (cols as f32 - 1.0)) / cols as f32).max(100.0);
        let mut chosen_accent: Option<AccentColor> = None;

        let draw_accent_card = |ui: &mut Ui, accent: AccentColor, name: &str| -> bool {
            let selected = app.settings.accent == accent;
            let swatch = theme::accent_swatch(accent);
            let (rect, resp) = ui.allocate_exact_size(vec2(ac_w, 46.0), Sense::click());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                let bg = if selected {
                    p.accent_dim
                } else if resp.hovered() {
                    p.bg_hover
                } else {
                    p.bg_3
                };
                let border = if selected { swatch } else { p.border };
                painter.rect_filled(rect, CornerRadius::same(RADIUS), bg);
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(RADIUS),
                    Stroke::new(if selected { 1.8_f32 } else { 1.0_f32 }, border),
                    StrokeKind::Inside,
                );
                let dot = pos2(rect.left() + 18.0, rect.center().y);
                painter.circle_filled(dot, 8.0, swatch);
                if selected {
                    let chk = Rect::from_center_size(dot, vec2(10.0, 10.0));
                    icons::paint(painter, chk, Icon::Check, Color32::WHITE);
                }
                painter.text(
                    pos2(dot.x + 16.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    name,
                    FontId::proportional(13.0),
                    p.text_0,
                );
            }
            resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
        };

        if two_col {
            for pair in accents.chunks(2) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    for (ac, name) in pair {
                        if draw_accent_card(ui, *ac, name) {
                            chosen_accent = Some(*ac);
                        }
                    }
                });
                ui.add_space(8.0);
            }
        } else {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                for (ac, name) in &accents {
                    if draw_accent_card(ui, *ac, name) {
                        chosen_accent = Some(*ac);
                    }
                }
            });
        }
        if let Some(ac) = chosen_accent {
            app.set_accent(ctx, ac);
        }
    });

    ui.add_space(14.0);

    theme::card_frame(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::section_header(ui, p, Icon::Globe, &lang_sec);
        theme::field_label(ui, p, &lang_lbl);
        ui.add_space(6.0);

        let mut picked_lang: Option<String> = None;
        ComboBox::from_id_salt("settings_lang_combo")
            .width(220.0)
            .selected_text(&selected_lang_name)
            .show_ui(ui, |ui| {
                for loc in &locales {
                    if ui
                        .selectable_label(current_lang == loc.code, &loc.name)
                        .clicked()
                    {
                        picked_lang = Some(loc.code.clone());
                    }
                }
            });
        if let Some(lang) = picked_lang {
            app.set_lang(lang);
        }
    });
}

fn render_status(ui: &mut Ui, p: &Palette, status: &StatusMessage) {
    let color = match status.kind {
        StatusKind::Ok => p.success,
        StatusKind::Err => p.danger,
        StatusKind::Neutral => p.text_1,
    };
    ui.label(RichText::new(&status.text).size(13.0).color(color));
}
