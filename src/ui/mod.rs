mod dashboard;
mod dm;
mod pack;
mod progress;
mod review;
mod settings;

use crate::app::{DisvokeApp, ModalState, Page, SettingsTab};
use crate::icons::{self, Icon};
use crate::store::{AccentColor, ThemeMode};
use crate::theme::{self, Palette, RADIUS};
use egui::{
    Align, Align2, Color32, ComboBox, Context, CornerRadius, CursorIcon, FontId, Frame, Id,
    Layout, Margin, Modal, Rect, ResizeDirection, ScrollArea, Sense, Stroke, StrokeKind, Ui,
    ViewportCommand, pos2, vec2,
};

pub fn render_app(app: &mut DisvokeApp, ctx: &Context) {
    let p = app.palette();
    handle_window_resize(ctx);
    render_titlebar(app, ctx, &p);
    render_sidebar(app, ctx, &p);

    let vp_w = ctx.viewport_rect().width();
    let pad_x = if vp_w < 860.0 { 16 } else { 28 };
    let pad_y = if vp_w < 860.0 { 16 } else { 24 };

    egui::CentralPanel::default()
        .frame(Frame::new().fill(p.bg_0).inner_margin(Margin::symmetric(pad_x, pad_y)))
        .show(ctx, |ui| match app.page {
            Page::Dashboard => dashboard::render(app, ui, &p),
            Page::Pack => pack::render(app, ui, ctx, &p),
            Page::DirectDm => dm::render(app, ui, ctx, &p),
            Page::Review => review::render(app, ui, &p),
            Page::Progress => progress::render(app, ui, &p),
            Page::Settings => settings::render(app, ui, ctx, &p),
        });

    render_modals(app, ctx, &p);
}

fn render_titlebar(app: &mut DisvokeApp, ctx: &Context, p: &Palette) {
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

    egui::TopBottomPanel::top("titlebar")
        .exact_height(42.0)
        .show_separator_line(false)
        .frame(
            Frame::new()
                .fill(p.bg_1)
                .inner_margin(Margin {
                    left: 16,
                    right: 10,
                    top: 0,
                    bottom: 0,
                }),
        )
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let vp = ctx.viewport_rect();
            ui.painter().line_segment(
                [pos2(vp.left(), rect.bottom()), pos2(vp.right(), rect.bottom())],
                Stroke::new(1.0_f32, p.border_subtle),
            );

            let drag_rect = Rect::from_min_max(
                rect.min,
                pos2((rect.right() - 150.0).max(rect.left()), rect.bottom()),
            );
            let drag_resp = ui.interact(drag_rect, Id::new("titlebar_drag"), Sense::click_and_drag());
            if drag_resp.drag_started_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }
            if drag_resp.double_clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            }

            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if let Some(tex) = &app.logo_texture {
                    ui.add(egui::Image::new((tex.id(), vec2(20.0, 20.0))));
                }
                ui.label(egui::RichText::new("Disvoke").size(13.5).strong().color(p.text_0));

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    if titlebar_btn(ui, p, Icon::X, true, app.i18n.t("titlebar.close")).clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                    let max_icon = if maximized { Icon::Restore } else { Icon::Maximize };
                    if titlebar_btn(ui, p, max_icon, false, app.i18n.t("titlebar.maximize")).clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                    }
                    if titlebar_btn(ui, p, Icon::Minimize, false, app.i18n.t("titlebar.minimize")).clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                    }

                    ui.add_space(6.0);
                    let theme_icon = if p.is_dark { Icon::Sun } else { Icon::Moon };
                    if titlebar_btn(ui, p, theme_icon, false, app.i18n.t("titlebar.toggleTheme")).clicked() {
                        let next = app.settings.theme.next();
                        app.set_theme(ctx, next);
                    }
                });
            });
        });
}

fn titlebar_btn(ui: &mut Ui, p: &Palette, icon: Icon, danger: bool, tooltip: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(30.0, 28.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let hovered = resp.hovered();
        let bg = if hovered {
            if danger { p.danger } else { p.bg_hover }
        } else {
            Color32::TRANSPARENT
        };
        let fg = if hovered {
            if danger { Color32::WHITE } else { p.text_0 }
        } else {
            p.text_1
        };
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(6), bg);
        icons::paint(painter, rect.shrink(7.0), icon, fg);
    }
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    if !tooltip.is_empty() {
        resp.on_hover_text(tooltip)
    } else {
        resp
    }
}

fn render_sidebar(app: &mut DisvokeApp, ctx: &Context, p: &Palette) {
    let compact = ctx.viewport_rect().width() < 900.0;
    let width = if compact { 66.0 } else { 226.0 };

    let sec_overview = app.i18n.t("nav.sectionOverview").to_string();
    let sec_source = app.i18n.t("nav.sectionSource").to_string();
    let sec_clean = app.i18n.t("nav.sectionClean").to_string();
    let sec_prefs = app.i18n.t("nav.sectionPrefs").to_string();

    let lbl_dash = app.i18n.t("nav.dashboard").to_string();
    let lbl_dm = app.i18n.t("nav.directDm").to_string();
    let lbl_pack = app.i18n.t("nav.pack").to_string();
    let lbl_review = app.i18n.t("nav.review").to_string();
    let lbl_progress = app.i18n.t("nav.progress").to_string();
    let lbl_settings = app.i18n.t("nav.settings").to_string();
    let lbl_connected = app.i18n.t("nav.connected").to_string();
    let lbl_not_connected = app.i18n.t("nav.notConnected").to_string();
    let lbl_running = app.i18n.t("nav.running").to_string();

    let selected_count = app.selected_ids.len();
    let review_badge = (selected_count > 0).then(|| theme::format_number(selected_count));
    let progress_badge = app.progress.running.then_some(lbl_running);

    egui::SidePanel::left("sidebar")
        .exact_width(width)
        .resizable(false)
        .show_separator_line(false)
        .frame(Frame::new().fill(p.bg_1).inner_margin(Margin::symmetric(10, 14)))
        .show(ctx, |ui| {
            render_sidebar_account_pill(app, ui, p, compact, &lbl_connected, &lbl_not_connected);
            ui.add_space(12.0);

            let footer_h = if compact { 126.0 } else { 132.0 };
            let nav_h = (ui.available_height() - footer_h).max(100.0);

            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), nav_h),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ScrollArea::vertical()
                        .id_salt("sidebar_nav_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 3.0;

                            sidebar_section(ui, p, compact, &sec_overview);
                            sidebar_item(app, ui, p, compact, Page::Dashboard, Icon::Dashboard, &lbl_dash, None);

                            ui.add_space(8.0);
                            sidebar_section(ui, p, compact, &sec_source);
                            sidebar_item(app, ui, p, compact, Page::DirectDm, Icon::Message, &lbl_dm, None);
                            sidebar_item(app, ui, p, compact, Page::Pack, Icon::Folder, &lbl_pack, None);

                            ui.add_space(8.0);
                            sidebar_section(ui, p, compact, &sec_clean);
                            sidebar_item(
                                app,
                                ui,
                                p,
                                compact,
                                Page::Review,
                                Icon::CheckSquare,
                                &lbl_review,
                                review_badge.as_deref(),
                            );
                            sidebar_item(
                                app,
                                ui,
                                p,
                                compact,
                                Page::Progress,
                                Icon::Activity,
                                &lbl_progress,
                                progress_badge.as_deref(),
                            );

                            ui.add_space(8.0);
                            sidebar_section(ui, p, compact, &sec_prefs);
                            sidebar_item(app, ui, p, compact, Page::Settings, Icon::Settings, &lbl_settings, None);
                        });
                },
            );

            ui.add_space(6.0);
            render_sidebar_footer(app, ctx, ui, p, compact);
        });
}

fn render_sidebar_account_pill(
    app: &mut DisvokeApp,
    ui: &mut Ui,
    p: &Palette,
    compact: bool,
    connected_lbl: &str,
    not_connected_lbl: &str,
) {
    let w = ui.available_width();
    let h = 40.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    let has_user = app.validated_user.is_some();
    let has_token = !app.settings.token.trim().is_empty();
    let dot_color = if has_user {
        p.success
    } else if has_token {
        p.accent
    } else {
        p.warn
    };

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if resp.hovered() { p.bg_hover } else { p.bg_2 };
        painter.rect_filled(rect, CornerRadius::same(RADIUS), bg);
        painter.rect_stroke(
            rect,
            CornerRadius::same(RADIUS),
            Stroke::new(1.0_f32, p.border),
            StrokeKind::Inside,
        );

        if compact {
            let av_rect = Rect::from_center_size(rect.center(), vec2(26.0, 26.0));
            painter.circle_filled(av_rect.center(), 13.0, p.bg_3);
            if let Some(tex) = &app.avatar_texture {
                egui::Image::new((tex.id(), vec2(26.0, 26.0))).paint_at(ui, av_rect);
            } else {
                icons::paint(painter, av_rect.shrink(5.0), Icon::Key, dot_color);
            }
            painter.circle_filled(pos2(av_rect.right() - 2.0, av_rect.bottom() - 2.0), 4.0, dot_color);
        } else {
            let dot_pos = pos2(rect.left() + 14.0, rect.center().y);
            painter.circle_filled(dot_pos, 4.5, dot_color);

            let label = if let Some(u) = &app.validated_user {
                u.username.clone()
            } else if has_token {
                connected_lbl.to_string()
            } else {
                not_connected_lbl.to_string()
            };
            let clip = Rect::from_min_max(
                pos2(rect.left() + 26.0, rect.top()),
                pos2(rect.right() - 10.0, rect.bottom()),
            );
            painter.with_clip_rect(clip).text(
                pos2(rect.left() + 26.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(12.5),
                if has_user || has_token { p.text_0 } else { p.text_1 },
            );
        }
    }

    let tip = app
        .validated_user
        .as_ref()
        .map(|u| format!("{} ({})", u.username, u.id))
        .unwrap_or_else(|| not_connected_lbl.to_string());
    if resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(tip).clicked() {
        app.open_settings_tab(SettingsTab::Account);
    }
}

fn sidebar_section(ui: &mut Ui, p: &Palette, compact: bool, title: &str) {
    if compact {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
        ui.painter().line_segment(
            [pos2(rect.left() + 8.0, rect.center().y), pos2(rect.right() - 8.0, rect.center().y)],
            Stroke::new(1.0_f32, p.border_subtle),
        );
    } else {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(title.to_uppercase())
                    .size(10.0)
                    .strong()
                    .color(p.text_2),
            );
        });
        ui.add_space(2.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn sidebar_item(
    app: &mut DisvokeApp,
    ui: &mut Ui,
    p: &Palette,
    compact: bool,
    page: Page,
    icon: Icon,
    label: &str,
    badge: Option<&str>,
) {
    let active = app.page == page;
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());

    if ui.is_rect_visible(rect) {
        let bg = if active {
            p.accent_dim
        } else if resp.hovered() {
            p.bg_hover
        } else {
            Color32::TRANSPARENT
        };
        let fg = if active {
            p.accent_text
        } else if resp.hovered() {
            p.text_0
        } else {
            p.text_1
        };

        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(RADIUS), bg);
        if active {
            let bar = Rect::from_min_size(pos2(rect.left() + 2.0, rect.top() + 8.0), vec2(3.0, rect.height() - 16.0));
            painter.rect_filled(bar, CornerRadius::same(2), p.accent);
        }

        if compact {
            let icon_rect = Rect::from_center_size(rect.center(), vec2(19.0, 19.0));
            icons::paint(painter, icon_rect, icon, fg);
            if badge.is_some() {
                painter.circle_filled(pos2(rect.right() - 9.0, rect.top() + 9.0), 3.5, p.accent);
            }
        } else {
            let icon_rect = Rect::from_min_size(pos2(rect.left() + 13.0, rect.center().y - 9.0), vec2(18.0, 18.0));
            icons::paint(painter, icon_rect, icon, fg);
            painter.text(
                pos2(rect.left() + 40.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(13.0),
                fg,
            );
            if let Some(b) = badge {
                let g = painter.layout_no_wrap(b.to_string(), FontId::proportional(10.5), Color32::WHITE);
                let bw = (g.size().x + 12.0).max(22.0);
                let brect = Rect::from_center_size(
                    pos2(rect.right() - bw * 0.5 - 8.0, rect.center().y),
                    vec2(bw, 18.0),
                );
                painter.rect_filled(brect, CornerRadius::same(9), p.accent);
                painter.galley(
                    pos2(brect.center().x - g.size().x * 0.5, brect.center().y - g.size().y * 0.5),
                    g,
                    Color32::WHITE,
                );
            }
        }
    }

    let mut resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    if compact {
        resp = resp.on_hover_text(label);
    }
    if resp.clicked() {
        app.page = page;
    }
}

fn render_sidebar_footer(
    app: &mut DisvokeApp,
    ctx: &Context,
    ui: &mut Ui,
    p: &Palette,
    compact: bool,
) {
    let accents = [
        (AccentColor::Orange, app.i18n.t("settings.accentOrange").to_string()),
        (AccentColor::Blue, app.i18n.t("settings.accentBlue").to_string()),
        (AccentColor::Green, app.i18n.t("settings.accentGreen").to_string()),
        (AccentColor::Red, app.i18n.t("settings.accentRed").to_string()),
    ];
    let locales = app.i18n.list_locales();
    let current_lang = app.settings.lang.clone();
    let current_lang_name = app.i18n.display_name(&current_lang).to_string();

    Frame::new()
        .fill(p.bg_2)
        .stroke(Stroke::new(1.0_f32, p.border))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::symmetric(8, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if compact {
                ui.vertical_centered(|ui| {
                    let theme_icon = match app.settings.theme {
                        ThemeMode::Light => Icon::Sun,
                        ThemeMode::Dark | ThemeMode::DarkOled => Icon::Moon,
                    };
                    let tip = app.i18n.t("titlebar.toggleTheme").to_string();
                    let (t_rect, t_resp) = ui.allocate_exact_size(vec2(30.0, 26.0), Sense::click());
                    if ui.is_rect_visible(t_rect) {
                        let bg = if t_resp.hovered() { p.bg_hover } else { p.bg_3 };
                        ui.painter().rect_filled(t_rect, CornerRadius::same(6), bg);
                        icons::paint(ui.painter(), t_rect.shrink(6.0), theme_icon, p.accent_text);
                    }
                    if t_resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(tip).clicked() {
                        let next = app.settings.theme.next();
                        app.set_theme(ctx, next);
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                        for (ac, name) in &accents[..2] {
                            accent_dot(app, ctx, ui, p, *ac, name, 15.0);
                        }
                    });
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                        for (ac, name) in &accents[2..] {
                            accent_dot(app, ctx, ui, p, *ac, name, 15.0);
                        }
                    });

                    ui.add_space(6.0);
                    let code_label = current_lang.to_uppercase();
                    let lang_tip = app.i18n.t("settings.languageLabel").to_string();
                    let (l_rect, l_resp) = ui.allocate_exact_size(vec2(30.0, 24.0), Sense::click());
                    if ui.is_rect_visible(l_rect) {
                        let bg = if l_resp.hovered() { p.bg_hover } else { p.bg_3 };
                        ui.painter().rect_filled(l_rect, CornerRadius::same(6), bg);
                        ui.painter().text(
                            l_rect.center(),
                            Align2::CENTER_CENTER,
                            &code_label,
                            FontId::proportional(10.5),
                            p.text_0,
                        );
                    }
                    if l_resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(lang_tip).clicked()
                        && !locales.is_empty()
                    {
                        let idx = locales.iter().position(|l| l.code == current_lang).unwrap_or(0);
                        let next = &locales[(idx + 1) % locales.len()];
                        app.set_lang(next.code.clone());
                    }
                });
            } else {
                let dark_lbl = app.i18n.t("settings.themeDark").to_string();
                let oled_lbl = app.i18n.t("settings.themeDarkOledShort").to_string();
                let light_lbl = app.i18n.t("settings.themeLight").to_string();
                let modes = [
                    (ThemeMode::Dark, dark_lbl),
                    (ThemeMode::DarkOled, oled_lbl),
                    (ThemeMode::Light, light_lbl),
                ];

                let avail = ui.available_width();
                let seg_w = ((avail - 8.0) / 3.0).max(40.0);
                let mut picked_theme: Option<ThemeMode> = None;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (mode, lbl) in &modes {
                        let selected = app.settings.theme == *mode;
                        let (rect, resp) = ui.allocate_exact_size(vec2(seg_w, 26.0), Sense::click());
                        if ui.is_rect_visible(rect) {
                            let painter = ui.painter();
                            let bg = if selected {
                                p.accent_dim
                            } else if resp.hovered() {
                                p.bg_hover
                            } else {
                                p.bg_3
                            };
                            let fg = if selected { p.accent_text } else { p.text_1 };
                            painter.rect_filled(rect, CornerRadius::same(6), bg);
                            if selected {
                                painter.rect_stroke(
                                    rect,
                                    CornerRadius::same(6),
                                    Stroke::new(1.0_f32, p.accent_border),
                                    StrokeKind::Inside,
                                );
                            }
                            painter.text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                lbl,
                                FontId::proportional(11.0),
                                fg,
                            );
                        }
                        if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                            picked_theme = Some(*mode);
                        }
                    }
                });
                if let Some(m) = picked_theme {
                    app.set_theme(ctx, m);
                }

                ui.add_space(8.0);

                let dot_size = 20.0;
                let dot_gap = 12.0;
                let row_w = dot_size * 4.0 + dot_gap * 3.0;
                let pad = ((avail - row_w) * 0.5).max(0.0);
                ui.horizontal(|ui| {
                    ui.add_space(pad);
                    ui.spacing_mut().item_spacing.x = dot_gap;
                    for (ac, name) in &accents {
                        accent_dot(app, ctx, ui, p, *ac, name, dot_size);
                    }
                });

                ui.add_space(8.0);

                let mut picked_lang: Option<String> = None;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let (g_slot, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());

                    let combo_w = (ui.available_width() - 2.0).max(80.0);
                    let combo_resp = ComboBox::from_id_salt("sidebar_lang_combo")
                        .width(combo_w)
                        .selected_text(&current_lang_name)
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

                    let globe_rect = Rect::from_center_size(
                        pos2(g_slot.center().x, combo_resp.response.rect.center().y),
                        vec2(16.0, 16.0),
                    );
                    icons::paint(ui.painter(), globe_rect, Icon::Globe, p.text_2);
                });
                if let Some(lang) = picked_lang {
                    app.set_lang(lang);
                }
            }
        });
}

fn accent_dot(
    app: &mut DisvokeApp,
    ctx: &Context,
    ui: &mut Ui,
    p: &Palette,
    accent: AccentColor,
    tooltip: &str,
    size: f32,
) {
    let selected = app.settings.accent == accent;
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    if ui.is_rect_visible(rect) {
        let color = theme::accent_swatch(accent);
        let painter = ui.painter();
        let outer_r = size * 0.5 - 0.5;
        let inner_r = outer_r - 2.5;
        if selected {
            painter.circle_stroke(rect.center(), outer_r, Stroke::new(1.8_f32, p.text_0));
        } else if resp.hovered() {
            painter.circle_stroke(rect.center(), outer_r, Stroke::new(1.2_f32, p.text_2));
        }
        painter.circle_filled(rect.center(), inner_r, color);
        if selected && size >= 18.0 {
            let chk = Rect::from_center_size(rect.center(), vec2(9.0, 9.0));
            icons::paint(painter, chk, Icon::Check, Color32::WHITE);
        }
    }
    if resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(tooltip).clicked() {
        app.set_accent(ctx, accent);
    }
}

fn render_modals(app: &mut DisvokeApp, ctx: &Context, p: &Palette) {
    match app.modal.clone() {
        ModalState::None => {}
        ModalState::NoTokenAlert => {
            let title = app.i18n.t("confirm.title").to_string();
            let msg = app.i18n.t("settings.noTokenAlert").to_string();
            let ok_label = app.i18n.t("dashboard.configureAccountBtn").to_string();
            let cancel_label = app.i18n.t("confirm.cancel").to_string();

            let resp = Modal::new(Id::new("modal_no_token")).show(ctx, |ui| {
                ui.set_max_width(420.0);
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                    icons::paint(ui.painter(), r, Icon::AlertTriangle, p.warn);
                    ui.label(egui::RichText::new(&title).size(16.0).strong().color(p.text_0));
                });
                ui.add_space(10.0);
                ui.label(egui::RichText::new(&msg).color(p.text_1));
                ui.add_space(18.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_button(ui, p, Some(Icon::Key), &ok_label, true).clicked() {
                        app.modal = ModalState::None;
                        app.open_settings_tab(SettingsTab::Account);
                    }
                    if theme::outline_button(ui, p, None, &cancel_label, true).clicked() {
                        app.modal = ModalState::None;
                    }
                });
            });
            if resp.should_close() {
                app.modal = ModalState::None;
            }
        }
        ModalState::ConfirmDeletion { count } => {
            let title = app.i18n.t("confirm.title").to_string();
            let body = app
                .i18n
                .fmt("confirm.message", &[("count", &theme::format_number(count))]);
            let delete_label = app.i18n.t("review.deleteBtn").to_string();
            let cancel_label = app.i18n.t("confirm.cancel").to_string();

            let resp = Modal::new(Id::new("modal_confirm_delete")).show(ctx, |ui| {
                ui.set_max_width(440.0);
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                    icons::paint(ui.painter(), r, Icon::AlertTriangle, p.danger);
                    ui.label(egui::RichText::new(&title).size(16.0).strong().color(p.text_0));
                });
                ui.add_space(10.0);
                ui.label(egui::RichText::new(&body).color(p.text_1));
                ui.add_space(18.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::danger_button(ui, p, Some(Icon::Trash), &delete_label, true).clicked() {
                        app.modal = ModalState::None;
                        app.start_deletion(ctx);
                    }
                    if theme::outline_button(ui, p, None, &cancel_label, true).clicked() {
                        app.modal = ModalState::None;
                    }
                });
            });
            if resp.should_close() {
                app.modal = ModalState::None;
            }
        }
    }
}

fn handle_window_resize(ctx: &Context) {
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    if maximized {
        return;
    }

    let rect = ctx.viewport_rect();
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };

    const EDGE: f32 = 5.0;
    let left = pos.x <= rect.left() + EDGE;
    let right = pos.x >= rect.right() - EDGE;
    let top = pos.y <= rect.top() + EDGE;
    let bottom = pos.y >= rect.bottom() - EDGE;

    let dir = match (left, right, top, bottom) {
        (true, _, true, _) => Some(ResizeDirection::NorthWest),
        (_, true, true, _) => Some(ResizeDirection::NorthEast),
        (true, _, _, true) => Some(ResizeDirection::SouthWest),
        (_, true, _, true) => Some(ResizeDirection::SouthEast),
        (true, _, _, _) => Some(ResizeDirection::West),
        (_, true, _, _) => Some(ResizeDirection::East),
        (_, _, true, _) => Some(ResizeDirection::North),
        (_, _, _, true) => Some(ResizeDirection::South),
        _ => None,
    };

    if let Some(dir) = dir {
        let cursor = match dir {
            ResizeDirection::North | ResizeDirection::South => CursorIcon::ResizeVertical,
            ResizeDirection::East | ResizeDirection::West => CursorIcon::ResizeHorizontal,
            ResizeDirection::NorthWest | ResizeDirection::SouthEast => CursorIcon::ResizeNwSe,
            ResizeDirection::NorthEast | ResizeDirection::SouthWest => CursorIcon::ResizeNeSw,
        };
        ctx.set_cursor_icon(cursor);
        if ctx.input(|i| i.pointer.primary_pressed()) {
            ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
    }
}
