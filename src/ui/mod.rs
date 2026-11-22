mod dashboard;
mod dm;
mod pack;
mod progress;
mod review;
mod settings;

use crate::app::{DisvokeApp, ModalState, Page};
use crate::theme::{self, ACCENT, ACCENT_DIM, BG_0, BG_1, BG_3, BG_HOVER, BORDER, DANGER, RADIUS, TEXT_0, TEXT_1};
use egui::{
    Align, Align2, Color32, Context, CornerRadius, CursorIcon, Frame, Id, Layout, Margin, Modal,
    Rect, ResizeDirection, Sense, Stroke, StrokeKind, Ui, ViewportCommand, pos2, vec2,
};

pub fn render_app(app: &mut DisvokeApp, ctx: &Context) {
    handle_window_resize(ctx);
    render_titlebar(app, ctx);
    render_sidebar(app, ctx);

    egui::CentralPanel::default()
        .frame(Frame::new().fill(BG_0).inner_margin(Margin::symmetric(32, 28)))
        .show(ctx, |ui| match app.page {
            Page::Dashboard => dashboard::render(app, ui),
            Page::Pack => pack::render(app, ui, ctx),
            Page::DirectDm => dm::render(app, ui, ctx),
            Page::Review => review::render(app, ui),
            Page::Progress => progress::render(app, ui),
            Page::Settings => settings::render(app, ui, ctx),
        });

    render_modals(app, ctx);
}

fn render_titlebar(app: &mut DisvokeApp, ctx: &Context) {
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

    egui::TopBottomPanel::top("titlebar")
        .exact_height(38.0)
        .frame(
            Frame::new()
                .fill(BG_1)
                .inner_margin(Margin {
                    left: 16,
                    right: 10,
                    top: 0,
                    bottom: 0,
                }),
        )
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(1.0_f32, BORDER),
            );

            let drag_rect = Rect::from_min_max(
                rect.min,
                pos2((rect.right() - 108.0).max(rect.left()), rect.bottom()),
            );
            let drag_resp = ui.interact(drag_rect, Id::new("titlebar_drag"), Sense::click_and_drag());
            if drag_resp.drag_started_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }
            if drag_resp.double_clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            }

            ui.horizontal_centered(|ui| {
                if let Some(tex) = &app.logo_texture {
                    ui.add(egui::Image::new((tex.id(), vec2(18.0, 18.0))));
                }
                ui.label(
                    egui::RichText::new("Disvoke")
                        .size(13.0)
                        .strong()
                        .color(TEXT_0),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    if titlebar_btn(ui, WindowBtn::Close, app.i18n.t("titlebar.close")).clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                    if titlebar_btn(ui, WindowBtn::Maximize, app.i18n.t("titlebar.maximize")).clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                    }
                    if titlebar_btn(ui, WindowBtn::Minimize, app.i18n.t("titlebar.minimize")).clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                    }
                });
            });
        });
}

enum WindowBtn {
    Minimize,
    Maximize,
    Close,
}

fn titlebar_btn(ui: &mut Ui, kind: WindowBtn, tooltip: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let hovered = resp.hovered();
        let bg = if hovered {
            match kind {
                WindowBtn::Close => DANGER,
                _ => BG_3,
            }
        } else {
            Color32::TRANSPARENT
        };
        let fg = if hovered {
            match kind {
                WindowBtn::Close => Color32::WHITE,
                _ => TEXT_0,
            }
        } else {
            TEXT_1
        };

        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(6), bg);
        let c = rect.center();
        let stroke = Stroke::new(1.4_f32, fg);
        match kind {
            WindowBtn::Minimize => {
                painter.line_segment([pos2(c.x - 5.0, c.y), pos2(c.x + 5.0, c.y)], stroke);
            }
            WindowBtn::Maximize => {
                let r = Rect::from_center_size(c, vec2(9.0, 9.0));
                painter.rect_stroke(r, CornerRadius::same(1), stroke, StrokeKind::Middle);
            }
            WindowBtn::Close => {
                painter.line_segment([pos2(c.x - 4.5, c.y - 4.5), pos2(c.x + 4.5, c.y + 4.5)], stroke);
                painter.line_segment([pos2(c.x + 4.5, c.y - 4.5), pos2(c.x - 4.5, c.y + 4.5)], stroke);
            }
        }
    }
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    if !tooltip.is_empty() {
        resp.on_hover_text(tooltip)
    } else {
        resp
    }
}

fn render_sidebar(app: &mut DisvokeApp, ctx: &Context) {
    let nav_items = [
        (Page::Dashboard, app.i18n.t("nav.dashboard").to_string()),
        (Page::Pack, app.i18n.t("nav.pack").to_string()),
        (Page::DirectDm, app.i18n.t("nav.directDm").to_string()),
        (Page::Review, app.i18n.t("nav.review").to_string()),
        (Page::Progress, app.i18n.t("nav.progress").to_string()),
        (Page::Settings, app.i18n.t("nav.settings").to_string()),
    ];

    egui::SidePanel::left("sidebar")
        .exact_width(190.0)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(BG_1)
                .inner_margin(Margin::symmetric(8, 12)),
        )
        .show(ctx, |ui| {
            let panel_rect = ui.max_rect();
            ui.painter().line_segment(
                [panel_rect.right_top(), panel_rect.right_bottom()],
                Stroke::new(1.0_f32, BORDER),
            );

            ui.spacing_mut().item_spacing.y = 4.0;
            for (page, label) in nav_items {
                let active = app.page == page;
                let width = ui.available_width();
                let (rect, resp) = ui.allocate_exact_size(vec2(width, 38.0), Sense::click());

                if ui.is_rect_visible(rect) {
                    let bg = if active {
                        ACCENT_DIM
                    } else if resp.hovered() {
                        BG_HOVER
                    } else {
                        Color32::TRANSPARENT
                    };
                    let fg = if active {
                        ACCENT
                    } else if resp.hovered() {
                        TEXT_0
                    } else {
                        TEXT_1
                    };

                    let painter = ui.painter();
                    painter.rect_filled(rect, CornerRadius::same(RADIUS), bg);

                    let icon_rect = Rect::from_min_size(
                        pos2(rect.left() + 12.0, rect.center().y - 9.0),
                        vec2(18.0, 18.0),
                    );
                    theme::paint_nav_icon(painter, icon_rect, page, fg);
                    painter.text(
                        pos2(rect.left() + 40.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        &label,
                        egui::FontId::proportional(13.0),
                        fg,
                    );
                }

                if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    app.page = page;
                }
            }
        });
}

fn render_modals(app: &mut DisvokeApp, ctx: &Context) {
    match app.modal.clone() {
        ModalState::None => {}
        ModalState::NoTokenAlert => {
            let title = app.i18n.t("confirm.title").to_string();
            let msg = app.i18n.t("settings.noTokenAlert").to_string();
            let ok_label = app.i18n.t("dashboard.settings").to_string();

            let resp = Modal::new(Id::new("modal_no_token")).show(ctx, |ui| {
                ui.set_max_width(420.0);
                ui.label(egui::RichText::new(&title).size(16.0).strong().color(theme::WARN));
                ui.add_space(10.0);
                ui.label(egui::RichText::new(&msg).color(TEXT_0));
                ui.add_space(18.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_button(ui, &ok_label, true).clicked() {
                        app.modal = ModalState::None;
                        app.page = Page::Settings;
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
            let cancel_label = app.i18n.t("titlebar.close").to_string();

            let resp = Modal::new(Id::new("modal_confirm_delete")).show(ctx, |ui| {
                ui.set_max_width(440.0);
                ui.label(egui::RichText::new(&title).size(16.0).strong().color(theme::WARN));
                ui.add_space(10.0);
                ui.label(egui::RichText::new(&body).color(TEXT_0));
                ui.add_space(18.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::danger_button(ui, &delete_label, true).clicked() {
                        app.modal = ModalState::None;
                        app.start_deletion(ctx);
                    }
                    if theme::outline_button(ui, &cancel_label, true).clicked() {
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
