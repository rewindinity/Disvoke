use crate::app::{DisvokeApp, Page, SettingsTab};
use crate::icons::{self, Icon};
use crate::theme::{self, Palette, RADIUS_LG, format_number};
use egui::{
    Align, Align2, CornerRadius, CursorIcon, FontId, Layout, Rect, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2,
};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, p: &Palette) {
    let title = app.i18n.t("dashboard.title").to_string();
    let subtitle = app.i18n.t("dashboard.subtitle").to_string();

    ScrollArea::vertical()
        .id_salt("dashboard_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            theme::centered_column(ui, 980.0, |ui| {
                theme::page_header(ui, p, Icon::Dashboard, &title, Some(&subtitle));
                ui.add_space(18.0);

                render_account_card(app, ui, p);
                ui.add_space(16.0);

                render_step_guide(app, ui, p);
                ui.add_space(20.0);

                render_method_cards(app, ui, p);
                ui.add_space(20.0);

                render_stats_grid(app, ui, p);
                ui.add_space(8.0);
            });
        });
}

fn render_account_card(app: &mut DisvokeApp, ui: &mut Ui, p: &Palette) {
    let not_logged = app.i18n.t("dashboard.notLoggedIn").to_string();
    let configure_sub = app.i18n.t("dashboard.configureToken").to_string();
    let connect_btn = app.i18n.t("dashboard.configureAccountBtn").to_string();
    let ready_badge = app.i18n.t("dashboard.accountReadyBadge").to_string();
    let setup_badge = app.i18n.t("dashboard.setupNeededBadge").to_string();

    let mut open_account = false;

    theme::card_frame(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let narrow = ui.available_width() < 580.0;

        let draw_identity = |ui: &mut Ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            let (av_rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
            let painter = ui.painter();
            painter.circle_filled(av_rect.center(), 24.0, p.bg_3);

            if let Some(tex) = &app.avatar_texture {
                egui::Image::new((tex.id(), vec2(48.0, 48.0))).paint_at(ui, av_rect);
            } else if let Some(user) = &app.validated_user {
                let initial = user
                    .username
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string())
                    .unwrap_or_else(|| "?".to_string());
                painter.text(
                    av_rect.center(),
                    Align2::CENTER_CENTER,
                    initial,
                    FontId::proportional(20.0),
                    p.text_0,
                );
            } else {
                icons::paint(painter, av_rect.shrink(13.0), Icon::Key, p.text_2);
            }
            painter.circle_stroke(
                av_rect.center(),
                24.0,
                Stroke::new(
                    2.0_f32,
                    if app.validated_user.is_some() { p.success } else { p.border },
                ),
            );

            ui.vertical(|ui| {
                ui.add_space(4.0);
                if let Some(user) = &app.validated_user {
                    ui.label(RichText::new(&user.username).size(15.5).strong().color(p.text_0));
                    ui.label(RichText::new(format!("ID: {}", user.id)).size(12.0).color(p.text_2));
                } else {
                    ui.label(RichText::new(&not_logged).size(15.5).strong().color(p.text_0));
                    ui.label(RichText::new(&configure_sub).size(12.5).color(p.text_2));
                }
            });
        };

        if narrow {
            ui.horizontal(|ui| draw_identity(ui));
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                if app.validated_user.is_some() {
                    theme::pill_badge(ui, &ready_badge, p.success_dim, p.success, Some(p.success));
                } else {
                    theme::pill_badge(ui, &setup_badge, p.warn_dim, p.warn, Some(p.warn));
                    if theme::primary_button(ui, p, Some(Icon::Key), &connect_btn, true).clicked() {
                        open_account = true;
                    }
                }
            });
        } else {
            ui.horizontal(|ui| {
                draw_identity(ui);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    if app.validated_user.is_some() {
                        theme::pill_badge(ui, &ready_badge, p.success_dim, p.success, Some(p.success));
                    } else {
                        if theme::primary_button(ui, p, Some(Icon::Key), &connect_btn, true).clicked() {
                            open_account = true;
                        }
                        theme::pill_badge(ui, &setup_badge, p.warn_dim, p.warn, Some(p.warn));
                    }
                });
            });
        }
    });

    if open_account {
        app.open_settings_tab(SettingsTab::Account);
    }
}

fn render_step_guide(app: &mut DisvokeApp, ui: &mut Ui, p: &Palette) {
    let step1_done = app.validated_user.is_some() || !app.settings.token.trim().is_empty();
    let step2_done = app.pack_data.is_some() || app.dm.fetched_info.is_some();
    let step3_ready = !app.selected_ids.is_empty();

    let steps = [
        (
            app.i18n.t("dashboard.step1Title").to_string(),
            app.i18n.t("dashboard.step1Desc").to_string(),
            step1_done,
            Icon::Key,
            Page::Settings,
        ),
        (
            app.i18n.t("dashboard.step2Title").to_string(),
            app.i18n.t("dashboard.step2Desc").to_string(),
            step2_done,
            Icon::Message,
            Page::DirectDm,
        ),
        (
            app.i18n.t("dashboard.step3Title").to_string(),
            app.i18n.t("dashboard.step3Desc").to_string(),
            step3_ready,
            Icon::CheckSquare,
            Page::Review,
        ),
    ];

    let avail = ui.available_width();
    let stacked = avail < 720.0;
    let gap = 10.0;
    let card_w = if stacked {
        avail
    } else {
        ((avail - gap * 2.0) / 3.0).floor().max(180.0)
    };
    let text_wrap_w = (card_w - 68.0).max(80.0);

    let step_heights: Vec<f32> = steps
        .iter()
        .map(|(title, desc, _, _, _)| {
            let tg = ui.painter().layout(
                title.clone(),
                FontId::proportional(13.0),
                p.text_0,
                text_wrap_w,
            );
            let dg = ui.painter().layout(
                desc.clone(),
                FontId::proportional(11.5),
                p.text_2,
                text_wrap_w,
            );
            (tg.size().y + 3.0 + dg.size().y + 24.0).max(66.0)
        })
        .collect();
    let shared_step_h = step_heights.iter().copied().fold(66.0_f32, f32::max);

    let mut target_page: Option<Page> = None;

    let draw_step = |ui: &mut Ui,
                     title: &str,
                     desc: &str,
                     done: bool,
                     icon: Icon,
                     page: Page,
                     h: f32|
     -> Option<Page> {
        let (rect, resp) = ui.allocate_exact_size(vec2(card_w, h), Sense::click());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let fill = if resp.hovered() { p.bg_hover } else { p.bg_2 };
            let border = if done { p.accent_border } else { p.border };
            painter.rect_filled(rect, CornerRadius::same(RADIUS_LG), fill);
            painter.rect_stroke(
                rect,
                CornerRadius::same(RADIUS_LG),
                Stroke::new(1.0_f32, border),
                StrokeKind::Inside,
            );

            let badge = Rect::from_min_size(
                pos2(rect.left() + 14.0, rect.center().y - 15.0),
                vec2(30.0, 30.0),
            );
            let badge_bg = if done { p.success_dim } else { p.accent_dim };
            let badge_fg = if done { p.success } else { p.accent_text };
            painter.rect_filled(badge, CornerRadius::same(8), badge_bg);
            icons::paint(
                painter,
                badge.shrink(7.0),
                if done { Icon::Check } else { icon },
                badge_fg,
            );

            let text_x = badge.right() + 12.0;
            let wrap_w = (rect.right() - text_x - 12.0).max(60.0);
            let title_g = painter.layout(
                title.to_string(),
                FontId::proportional(13.0),
                p.text_0,
                wrap_w,
            );
            let desc_g = painter.layout(
                desc.to_string(),
                FontId::proportional(11.5),
                p.text_2,
                wrap_w,
            );
            let title_h = title_g.size().y;
            let total_text_h = title_h + 3.0 + desc_g.size().y;
            let start_y = rect.center().y - total_text_h * 0.5;
            let clip = Rect::from_min_max(
                pos2(text_x, rect.top() + 4.0),
                pos2(rect.right() - 8.0, rect.bottom() - 4.0),
            );
            let cp = painter.with_clip_rect(clip);
            cp.galley(pos2(text_x, start_y), title_g, p.text_0);
            cp.galley(pos2(text_x, start_y + title_h + 3.0), desc_g, p.text_2);
        }
        if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            Some(page)
        } else {
            None
        }
    };

    if stacked {
        for ((title, desc, done, icon, page), h) in steps.into_iter().zip(step_heights) {
            if let Some(pg) = draw_step(ui, &title, &desc, done, icon, page, h) {
                target_page = Some(pg);
            }
            ui.add_space(6.0);
        }
    } else {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (title, desc, done, icon, page) in steps {
                if let Some(pg) = draw_step(ui, &title, &desc, done, icon, page, shared_step_h) {
                    target_page = Some(pg);
                }
            }
        });
    }

    if let Some(pg) = target_page {
        if pg == Page::Settings {
            app.open_settings_tab(SettingsTab::Account);
        } else {
            app.page = pg;
        }
    }
}

struct MethodCardSpec<'a> {
    icon: Icon,
    title: &'a str,
    badge: &'a str,
    desc: &'a str,
    cta: &'a str,
    primary: bool,
    page: Page,
}

fn measure_method_card(ui: &Ui, p: &Palette, card_w: f32, spec: &MethodCardSpec<'_>) -> f32 {
    let inner_w = (card_w - 40.0).max(120.0);
    let title_g = ui.painter().layout(
        spec.title.to_string(),
        FontId::proportional(15.5),
        p.text_0,
        inner_w,
    );
    let desc_g = ui.painter().layout(
        spec.desc.to_string(),
        FontId::proportional(12.5),
        p.text_1,
        inner_w,
    );
    18.0 + 34.0 + 12.0 + title_g.size().y + 6.0 + desc_g.size().y + 18.0 + 36.0 + 18.0
}

fn paint_method_card(
    ui: &mut Ui,
    p: &Palette,
    rect: Rect,
    spec: &MethodCardSpec<'_>,
) -> bool {
    let inner_w = (rect.width() - 40.0).max(120.0);
    let hovered = ui.rect_contains_pointer(rect);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let border = if hovered {
            p.accent
        } else if spec.primary {
            p.accent_border
        } else {
            p.border
        };
        painter.rect_filled(rect, CornerRadius::same(RADIUS_LG), p.bg_2);
        painter.rect_stroke(
            rect,
            CornerRadius::same(RADIUS_LG),
            Stroke::new(1.0_f32, border),
            StrokeKind::Inside,
        );

        let left = rect.left() + 20.0;
        let top = rect.top() + 18.0;

        let icon_rect = Rect::from_min_size(pos2(left, top), vec2(34.0, 34.0));
        painter.rect_filled(icon_rect, CornerRadius::same(9), p.accent_dim);
        icons::paint(painter, icon_rect.shrink(8.0), spec.icon, p.accent_text);

        let badge_fg = if spec.primary { p.accent_text } else { p.text_1 };
        let badge_bg = if spec.primary { p.accent_dim } else { p.bg_3 };
        let badge_g =
            painter.layout_no_wrap(spec.badge.to_string(), FontId::proportional(11.5), badge_fg);
        let badge_size = badge_g.size() + vec2(20.0, 8.0);
        let badge_rect = Rect::from_center_size(
            pos2(rect.right() - 20.0 - badge_size.x * 0.5, icon_rect.center().y),
            badge_size,
        );
        let badge_rad = CornerRadius::same((badge_rect.height() * 0.5).round() as u8);
        painter.rect_filled(badge_rect, badge_rad, badge_bg);
        painter.galley(
            pos2(
                badge_rect.center().x - badge_g.size().x * 0.5,
                badge_rect.center().y - badge_g.size().y * 0.5,
            ),
            badge_g,
            badge_fg,
        );

        let title_y = icon_rect.bottom() + 12.0;
        let title_g = painter.layout(
            spec.title.to_string(),
            FontId::proportional(15.5),
            p.text_0,
            inner_w,
        );
        let title_h = title_g.size().y;
        painter.galley(pos2(left, title_y), title_g, p.text_0);

        let desc_y = title_y + title_h + 6.0;
        let desc_g = painter.layout(
            spec.desc.to_string(),
            FontId::proportional(12.5),
            p.text_1,
            inner_w,
        );
        painter.galley(pos2(left, desc_y), desc_g, p.text_1);
    }

    let btn_rect = Rect::from_min_size(
        pos2(rect.left() + 20.0, rect.bottom() - 18.0 - 36.0),
        vec2(inner_w, 36.0),
    );
    let mut btn_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(btn_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let resp = if spec.primary {
        theme::primary_button(&mut btn_ui, p, Some(Icon::ArrowRight), spec.cta, true)
    } else {
        theme::outline_button(&mut btn_ui, p, Some(Icon::ArrowRight), spec.cta, true)
    };
    resp.clicked()
}

fn render_method_cards(app: &mut DisvokeApp, ui: &mut Ui, p: &Palette) {
    let heading = app.i18n.t("dashboard.chooseMethodTitle").to_string();
    ui.label(RichText::new(heading).size(14.0).strong().color(p.text_1));
    ui.add_space(8.0);

    let dm_title = app.i18n.t("dashboard.dmCardTitle").to_string();
    let dm_badge = app.i18n.t("dashboard.dmCardBadge").to_string();
    let dm_desc = app.i18n.t("dashboard.dmCardDesc").to_string();
    let dm_cta = app.i18n.t("dashboard.directDm").to_string();

    let pack_title = app.i18n.t("dashboard.packCardTitle").to_string();
    let pack_badge = app.i18n.t("dashboard.packCardBadge").to_string();
    let pack_desc = app.i18n.t("dashboard.packCardDesc").to_string();
    let pack_cta = app.i18n.t("dashboard.loadPack").to_string();

    let dm_spec = MethodCardSpec {
        icon: Icon::Message,
        title: &dm_title,
        badge: &dm_badge,
        desc: &dm_desc,
        cta: &dm_cta,
        primary: true,
        page: Page::DirectDm,
    };
    let pack_spec = MethodCardSpec {
        icon: Icon::Folder,
        title: &pack_title,
        badge: &pack_badge,
        desc: &pack_desc,
        cta: &pack_cta,
        primary: false,
        page: Page::Pack,
    };

    let avail = ui.available_width();
    let stacked = avail < 620.0;
    let gap = 14.0;
    let card_w = if stacked {
        avail
    } else {
        ((avail - gap) * 0.5).floor().max(220.0)
    };

    let dm_h = measure_method_card(ui, p, card_w, &dm_spec);
    let pack_h = measure_method_card(ui, p, card_w, &pack_spec);

    let mut open_page: Option<Page> = None;

    if stacked {
        let (r1, _) = ui.allocate_exact_size(vec2(card_w, dm_h), Sense::hover());
        if paint_method_card(ui, p, r1, &dm_spec) {
            open_page = Some(dm_spec.page);
        }
        ui.add_space(12.0);
        let (r2, _) = ui.allocate_exact_size(vec2(card_w, pack_h), Sense::hover());
        if paint_method_card(ui, p, r2, &pack_spec) {
            open_page = Some(pack_spec.page);
        }
    } else {
        let shared_h = dm_h.max(pack_h);
        let (row_rect, _) = ui.allocate_exact_size(vec2(avail, shared_h), Sense::hover());
        let r1 = Rect::from_min_size(row_rect.min, vec2(card_w, shared_h));
        let r2 = Rect::from_min_size(
            pos2(row_rect.left() + card_w + gap, row_rect.top()),
            vec2(card_w, shared_h),
        );
        if paint_method_card(ui, p, r1, &dm_spec) {
            open_page = Some(dm_spec.page);
        }
        if paint_method_card(ui, p, r2, &pack_spec) {
            open_page = Some(pack_spec.page);
        }
    }

    if let Some(pg) = open_page {
        app.page = pg;
    }
}

fn render_stats_grid(app: &DisvokeApp, ui: &mut Ui, p: &Palette) {
    let (total_str, servers_str, dms_str) = if let Some(pack) = &app.pack_data {
        (
            format_number(pack.total_messages),
            format_number(pack.servers.len()),
            format_number(pack.dms.len()),
        )
    } else if let Some(dm) = &app.dm.fetched_info {
        (
            format_number(dm.total_fetched),
            "0".to_string(),
            "1".to_string(),
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
    let avail = ui.available_width();
    let cols = if avail < 360.0 {
        1
    } else if avail < 700.0 {
        2
    } else {
        4
    };
    let card_w = ((avail - gap * (cols as f32 - 1.0)) / cols as f32).max(120.0);

    let paint_card = |ui: &mut Ui, value: &str, label: &str, accent: bool| {
        let (rect, _) = ui.allocate_exact_size(vec2(card_w, 88.0), Sense::hover());
        let painter = ui.painter();
        let fill = if accent { p.accent_dim } else { p.bg_2 };
        let stroke = Stroke::new(1.0_f32, if accent { p.accent_border } else { p.border });
        painter.rect_filled(rect, CornerRadius::same(RADIUS_LG), fill);
        painter.rect_stroke(rect, CornerRadius::same(RADIUS_LG), stroke, StrokeKind::Inside);

        let num_color = if accent { p.accent_text } else { p.text_0 };
        painter.text(
            pos2(rect.center().x, rect.top() + 31.0),
            Align2::CENTER_CENTER,
            value,
            FontId::proportional(25.0),
            num_color,
        );
        let lbl_g = painter.layout(
            label.to_uppercase(),
            FontId::proportional(10.5),
            p.text_2,
            (card_w - 20.0).max(60.0),
        );
        painter.galley(
            pos2(
                rect.center().x - lbl_g.size().x * 0.5,
                rect.bottom() - 22.0 - lbl_g.size().y * 0.5,
            ),
            lbl_g,
            p.text_2,
        );
    };

    for chunk in cards.chunks(cols) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (v, l, a) in chunk {
                paint_card(ui, v, l, *a);
            }
        });
        if cols < 4 {
            ui.add_space(gap);
        }
    }
}
