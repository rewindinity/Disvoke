use crate::icons::{self, Icon};
use crate::store::{AccentColor, ThemeMode};
use egui::{
    Align2, Color32, Context, CornerRadius, CursorIcon, FontData, FontDefinitions, FontFamily,
    FontId, Frame, Margin, Rect, Response, Sense, Stroke, StrokeKind, TextFormat, TextStyle, Ui,
    Vec2, Visuals, pos2, text::LayoutJob, vec2,
};
use std::sync::Arc;

pub const RADIUS: u8 = 8;
pub const RADIUS_LG: u8 = 12;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub is_dark: bool,
    pub bg_0: Color32,
    pub bg_1: Color32,
    pub bg_2: Color32,
    pub bg_3: Color32,
    pub bg_hover: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_text: Color32,
    pub accent_dim: Color32,
    pub accent_border: Color32,
    pub danger: Color32,
    pub danger_hover: Color32,
    pub danger_dim: Color32,
    pub success: Color32,
    pub success_dim: Color32,
    pub warn: Color32,
    pub warn_dim: Color32,
    pub text_0: Color32,
    pub text_1: Color32,
    pub text_2: Color32,
}

impl Palette {
    pub fn from_config(mode: ThemeMode, accent: AccentColor) -> Self {
        let (ac, ac_hover, ac_text, ac_dim, ac_border) = accent_colors(mode, accent);
        match mode {
            ThemeMode::Dark => Self {
                is_dark: true,
                bg_0: Color32::from_rgb(0x0f, 0x11, 0x15),
                bg_1: Color32::from_rgb(0x15, 0x18, 0x1e),
                bg_2: Color32::from_rgb(0x1c, 0x20, 0x28),
                bg_3: Color32::from_rgb(0x25, 0x2a, 0x35),
                bg_hover: Color32::from_rgb(0x2e, 0x34, 0x42),
                border: Color32::from_rgb(0x2c, 0x32, 0x40),
                border_subtle: Color32::from_rgb(0x23, 0x28, 0x34),
                accent: ac,
                accent_hover: ac_hover,
                accent_text: ac_text,
                accent_dim: ac_dim,
                accent_border: ac_border,
                danger: Color32::from_rgb(0xef, 0x44, 0x44),
                danger_hover: Color32::from_rgb(0xdc, 0x26, 0x26),
                danger_dim: Color32::from_rgba_unmultiplied(239, 68, 68, 28),
                success: Color32::from_rgb(0x22, 0xc5, 0x5e),
                success_dim: Color32::from_rgba_unmultiplied(34, 197, 94, 24),
                warn: Color32::from_rgb(0xf5, 0x9e, 0x0b),
                warn_dim: Color32::from_rgba_unmultiplied(245, 158, 11, 26),
                text_0: Color32::from_rgb(0xf3, 0xf4, 0xf6),
                text_1: Color32::from_rgb(0xb8, 0xbe, 0xc9),
                text_2: Color32::from_rgb(0x7c, 0x84, 0x94),
            },
            ThemeMode::DarkOled => Self {
                is_dark: true,
                bg_0: Color32::from_rgb(0x00, 0x00, 0x00),
                bg_1: Color32::from_rgb(0x05, 0x05, 0x07),
                bg_2: Color32::from_rgb(0x0b, 0x0c, 0x10),
                bg_3: Color32::from_rgb(0x13, 0x15, 0x1c),
                bg_hover: Color32::from_rgb(0x1c, 0x1f, 0x29),
                border: Color32::from_rgb(0x1e, 0x22, 0x2d),
                border_subtle: Color32::from_rgb(0x14, 0x16, 0x1e),
                accent: ac,
                accent_hover: ac_hover,
                accent_text: ac_text,
                accent_dim: ac_dim,
                accent_border: ac_border,
                danger: Color32::from_rgb(0xf8, 0x45, 0x45),
                danger_hover: Color32::from_rgb(0xdc, 0x26, 0x26),
                danger_dim: Color32::from_rgba_unmultiplied(239, 68, 68, 26),
                success: Color32::from_rgb(0x22, 0xc5, 0x5e),
                success_dim: Color32::from_rgba_unmultiplied(34, 197, 94, 22),
                warn: Color32::from_rgb(0xf5, 0x9e, 0x0b),
                warn_dim: Color32::from_rgba_unmultiplied(245, 158, 11, 24),
                text_0: Color32::from_rgb(0xf5, 0xf6, 0xf8),
                text_1: Color32::from_rgb(0xab, 0xb2, 0xbf),
                text_2: Color32::from_rgb(0x6c, 0x73, 0x82),
            },
            ThemeMode::Light => Self {
                is_dark: false,
                bg_0: Color32::from_rgb(0xf4, 0xf6, 0xf9),
                bg_1: Color32::from_rgb(0xea, 0xed, 0xf2),
                bg_2: Color32::from_rgb(0xff, 0xff, 0xff),
                bg_3: Color32::from_rgb(0xee, 0xf1, 0xf6),
                bg_hover: Color32::from_rgb(0xe2, 0xe6, 0xee),
                border: Color32::from_rgb(0xd4, 0xd9, 0xe3),
                border_subtle: Color32::from_rgb(0xe4, 0xe7, 0xef),
                accent: ac,
                accent_hover: ac_hover,
                accent_text: ac_text,
                accent_dim: ac_dim,
                accent_border: ac_border,
                danger: Color32::from_rgb(0xdc, 0x26, 0x26),
                danger_hover: Color32::from_rgb(0xb9, 0x1c, 0x1c),
                danger_dim: Color32::from_rgba_unmultiplied(220, 38, 38, 22),
                success: Color32::from_rgb(0x15, 0x80, 0x3d),
                success_dim: Color32::from_rgba_unmultiplied(21, 128, 61, 20),
                warn: Color32::from_rgb(0xb4, 0x53, 0x09),
                warn_dim: Color32::from_rgba_unmultiplied(180, 83, 9, 22),
                text_0: Color32::from_rgb(0x11, 0x18, 0x27),
                text_1: Color32::from_rgb(0x4b, 0x55, 0x63),
                text_2: Color32::from_rgb(0x6b, 0x72, 0x80),
            },
        }
    }
}

pub fn accent_swatch(accent: AccentColor) -> Color32 {
    match accent {
        AccentColor::Orange => Color32::from_rgb(0xf9, 0x73, 0x16),
        AccentColor::Blue => Color32::from_rgb(0x3b, 0x82, 0xf6),
        AccentColor::Green => Color32::from_rgb(0x22, 0xc5, 0x5e),
        AccentColor::Red => Color32::from_rgb(0xef, 0x44, 0x44),
    }
}

fn accent_colors(mode: ThemeMode, accent: AccentColor) -> (Color32, Color32, Color32, Color32, Color32) {
    match (mode, accent) {
        (ThemeMode::Dark | ThemeMode::DarkOled, AccentColor::Orange) => (
            Color32::from_rgb(0xf9, 0x73, 0x16),
            Color32::from_rgb(0xea, 0x58, 0x0c),
            Color32::from_rgb(0xfb, 0x92, 0x3c),
            Color32::from_rgba_unmultiplied(249, 115, 22, 34),
            Color32::from_rgba_unmultiplied(249, 115, 22, 115),
        ),
        (ThemeMode::Light, AccentColor::Orange) => (
            Color32::from_rgb(0xea, 0x58, 0x0c),
            Color32::from_rgb(0xc2, 0x41, 0x0c),
            Color32::from_rgb(0xc2, 0x41, 0x0c),
            Color32::from_rgba_unmultiplied(234, 88, 12, 26),
            Color32::from_rgba_unmultiplied(234, 88, 12, 105),
        ),
        (ThemeMode::Dark | ThemeMode::DarkOled, AccentColor::Blue) => (
            Color32::from_rgb(0x3b, 0x82, 0xf6),
            Color32::from_rgb(0x25, 0x63, 0xeb),
            Color32::from_rgb(0x60, 0xa5, 0xfa),
            Color32::from_rgba_unmultiplied(59, 130, 246, 34),
            Color32::from_rgba_unmultiplied(59, 130, 246, 115),
        ),
        (ThemeMode::Light, AccentColor::Blue) => (
            Color32::from_rgb(0x25, 0x63, 0xeb),
            Color32::from_rgb(0x1d, 0x4e, 0xd8),
            Color32::from_rgb(0x1d, 0x4e, 0xd8),
            Color32::from_rgba_unmultiplied(37, 99, 235, 26),
            Color32::from_rgba_unmultiplied(37, 99, 235, 105),
        ),
        (ThemeMode::Dark | ThemeMode::DarkOled, AccentColor::Green) => (
            Color32::from_rgb(0x22, 0xc5, 0x5e),
            Color32::from_rgb(0x16, 0xa3, 0x4a),
            Color32::from_rgb(0x4a, 0xde, 0x80),
            Color32::from_rgba_unmultiplied(34, 197, 94, 34),
            Color32::from_rgba_unmultiplied(34, 197, 94, 115),
        ),
        (ThemeMode::Light, AccentColor::Green) => (
            Color32::from_rgb(0x16, 0xa3, 0x4a),
            Color32::from_rgb(0x15, 0x80, 0x3d),
            Color32::from_rgb(0x15, 0x80, 0x3d),
            Color32::from_rgba_unmultiplied(22, 163, 74, 26),
            Color32::from_rgba_unmultiplied(22, 163, 74, 105),
        ),
        (ThemeMode::Dark | ThemeMode::DarkOled, AccentColor::Red) => (
            Color32::from_rgb(0xef, 0x44, 0x44),
            Color32::from_rgb(0xdc, 0x26, 0x26),
            Color32::from_rgb(0xf8, 0x71, 0x71),
            Color32::from_rgba_unmultiplied(239, 68, 68, 34),
            Color32::from_rgba_unmultiplied(239, 68, 68, 115),
        ),
        (ThemeMode::Light, AccentColor::Red) => (
            Color32::from_rgb(0xdc, 0x26, 0x26),
            Color32::from_rgb(0xb9, 0x1c, 0x1c),
            Color32::from_rgb(0xb9, 0x1c, 0x1c),
            Color32::from_rgba_unmultiplied(220, 38, 38, 26),
            Color32::from_rgba_unmultiplied(220, 38, 38, 105),
        ),
    }
}

pub fn configure_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    let fallback_groups: &[&[&str]] = &[
        &[
            "/usr/share/fonts/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ],
        &[
            "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
            "C:\\Windows\\Fonts\\seguisym.ttf",
            "/System/Library/Fonts/Apple Symbols.ttf",
        ],
    ];

    for (idx, group) in fallback_groups.iter().enumerate() {
        for path in *group {
            if let Ok(bytes) = std::fs::read(path) {
                let name = format!("sys_fallback_{idx}");
                fonts
                    .font_data
                    .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .push(name.clone());
                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .push(name);
                break;
            }
        }
    }

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &Context, p: &Palette) {
    let mut visuals = if p.is_dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.panel_fill = p.bg_0;
    visuals.window_fill = p.bg_2;
    visuals.faint_bg_color = p.bg_1;
    visuals.extreme_bg_color = p.bg_3;
    visuals.code_bg_color = p.bg_3;
    visuals.override_text_color = Some(p.text_0);
    visuals.hyperlink_color = p.accent_text;
    visuals.warn_fg_color = p.warn;
    visuals.error_fg_color = p.danger;
    visuals.window_corner_radius = CornerRadius::same(RADIUS_LG);
    visuals.window_stroke = Stroke::new(1.0_f32, p.border);
    visuals.menu_corner_radius = CornerRadius::same(RADIUS);

    visuals.selection.bg_fill = p.accent;
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

    visuals.widgets.noninteractive.bg_fill = p.bg_2;
    visuals.widgets.noninteractive.weak_bg_fill = p.bg_2;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text_1);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS);

    visuals.widgets.inactive.bg_fill = p.bg_3;
    visuals.widgets.inactive.weak_bg_fill = p.bg_3;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text_0);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(RADIUS);

    visuals.widgets.hovered.bg_fill = p.bg_hover;
    visuals.widgets.hovered.weak_bg_fill = p.bg_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.text_0);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(RADIUS);

    visuals.widgets.active.bg_fill = p.accent;
    visuals.widgets.active.weak_bg_fill = p.accent;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
    visuals.widgets.active.corner_radius = CornerRadius::same(RADIUS);

    visuals.widgets.open.bg_fill = p.bg_3;
    visuals.widgets.open.weak_bg_fill = p.bg_3;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, p.accent);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, p.text_0);
    visuals.widgets.open.corner_radius = CornerRadius::same(RADIUS);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(14.0, 8.0);
    style.spacing.text_edit_width = 320.0;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.handle_min_length = 28.0;

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(22.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(13.5, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.0, FontFamily::Monospace),
    );

    ctx.set_style(style);
}

pub fn card_frame(p: &Palette) -> Frame {
    Frame::new()
        .fill(p.bg_2)
        .stroke(Stroke::new(1.0_f32, p.border))
        .corner_radius(CornerRadius::same(RADIUS_LG))
        .inner_margin(Margin::symmetric(22, 18))
}

pub fn inset_frame(p: &Palette) -> Frame {
    Frame::new()
        .fill(p.bg_1)
        .stroke(Stroke::new(1.0_f32, p.border))
        .corner_radius(CornerRadius::same(RADIUS_LG))
        .inner_margin(Margin::same(10))
}

pub fn centered_column(ui: &mut Ui, max_width: f32, add_contents: impl FnOnce(&mut Ui)) {
    let avail = ui.available_width();
    let target = avail.min(max_width);
    let pad = ((avail - target) * 0.5).max(0.0);
    let prev_x = ui.spacing().item_spacing.x;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if pad > 0.0 {
            ui.add_space(pad);
        }
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.x = prev_x;
            ui.set_width(target);
            add_contents(ui);
        });
    });
}

pub fn page_header(ui: &mut Ui, p: &Palette, icon: Icon, title: &str, subtitle: Option<&str>) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        let (icon_box, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(icon_box, CornerRadius::same(10), p.accent_dim);
        painter.rect_stroke(
            icon_box,
            CornerRadius::same(10),
            Stroke::new(1.0_f32, p.accent_border),
            StrokeKind::Inside,
        );
        icons::paint(painter, icon_box.shrink(9.0), icon, p.accent_text);

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(egui::RichText::new(title).size(21.0).strong().color(p.text_0));
            if let Some(sub) = subtitle
                && !sub.is_empty()
            {
                ui.label(egui::RichText::new(sub).size(13.0).color(p.text_1));
            }
        });
    });
}

pub fn section_header(ui: &mut Ui, p: &Palette, icon: Icon, title: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (rect, _) = ui.allocate_exact_size(vec2(17.0, 17.0), Sense::hover());
        icons::paint(ui.painter(), rect, icon, p.accent_text);
        ui.label(egui::RichText::new(title).size(14.5).strong().color(p.text_0));
    });
    ui.add_space(10.0);
}

pub fn field_label(ui: &mut Ui, p: &Palette, text: &str) {
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .size(11.0)
            .strong()
            .color(p.text_1),
    );
}

pub fn field_hint(ui: &mut Ui, p: &Palette, text: &str) {
    if !text.is_empty() {
        ui.label(egui::RichText::new(text).size(11.5).color(p.text_2));
    }
}

pub fn pill_badge(ui: &mut Ui, text: &str, bg: Color32, fg: Color32, border: Option<Color32>) {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), FontId::proportional(11.5), fg);
    let desired = galley.size() + vec2(20.0, 8.0);
    let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
    if ui.is_rect_visible(rect) {
        let radius = CornerRadius::same((rect.height() * 0.5).round() as u8);
        let painter = ui.painter();
        painter.rect_filled(rect, radius, bg);
        if let Some(b) = border {
            painter.rect_stroke(rect, radius, Stroke::new(1.0_f32, b), StrokeKind::Inside);
        }
        painter.galley(
            pos2(
                rect.center().x - galley.size().x * 0.5,
                rect.center().y - galley.size().y * 0.5,
            ),
            galley,
            fg,
        );
    }
}

pub fn help_drawer(
    ui: &mut Ui,
    p: &Palette,
    id: &str,
    title: &str,
    open: &mut bool,
    steps: &[&str],
) {
    let header_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(header_w, 36.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if resp.hovered() { p.bg_hover } else { p.bg_3 };
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(RADIUS), fill);
        painter.rect_stroke(
            rect,
            CornerRadius::same(RADIUS),
            Stroke::new(1.0_f32, p.border),
            StrokeKind::Inside,
        );
        let icon_rect = Rect::from_min_size(pos2(rect.left() + 10.0, rect.center().y - 8.0), vec2(16.0, 16.0));
        icons::paint(painter, icon_rect, Icon::HelpCircle, p.accent_text);
        painter.text(
            pos2(rect.left() + 34.0, rect.center().y),
            Align2::LEFT_CENTER,
            title,
            FontId::proportional(12.5),
            p.text_0,
        );
        let chev_rect = Rect::from_min_size(pos2(rect.right() - 24.0, rect.center().y - 7.0), vec2(14.0, 14.0));
        icons::paint(
            painter,
            chev_rect,
            if *open { Icon::ChevronDown } else { Icon::ChevronRight },
            p.text_2,
        );
    }
    if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
        *open = !*open;
    }

    if *open {
        ui.add_space(6.0);
        Frame::new()
            .fill(p.bg_1)
            .stroke(Stroke::new(1.0_f32, p.border_subtle))
            .corner_radius(CornerRadius::same(RADIUS))
            .inner_margin(Margin::symmetric(14, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.push_id(id, |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    for (idx, step) in steps.iter().enumerate() {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}.", idx + 1))
                                    .size(12.5)
                                    .strong()
                                    .color(p.accent_text),
                            );
                            ui.label(egui::RichText::new(*step).size(12.5).color(p.text_1));
                        });
                    }
                });
            });
    }
}

struct ButtonStyle {
    bg: Color32,
    bg_hover: Color32,
    text_color: Color32,
    stroke: Stroke,
    padding: Vec2,
    font_size: f32,
    icon_size: f32,
}

fn styled_button(
    ui: &mut Ui,
    label: &str,
    icon: Option<Icon>,
    style: ButtonStyle,
    enabled: bool,
) -> Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::proportional(style.font_size),
        style.text_color,
    );
    let icon_gap = if icon.is_some() && !label.is_empty() { 7.0 } else { 0.0 };
    let icon_w = if icon.is_some() { style.icon_size } else { 0.0 };
    let content_w = icon_w + icon_gap + galley.size().x;
    let content_h = galley.size().y.max(style.icon_size);
    let desired = vec2(content_w, content_h) + style.padding * 2.0;

    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(desired, sense);

    if ui.is_rect_visible(rect) {
        let fill = if !enabled {
            style.bg.gamma_multiply(0.45)
        } else if resp.hovered() {
            style.bg_hover
        } else {
            style.bg
        };
        let fg = if enabled {
            style.text_color
        } else {
            style.text_color.gamma_multiply(0.55)
        };
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(RADIUS), fill);
        if style.stroke.width > 0.0 {
            let st = if enabled {
                style.stroke
            } else {
                Stroke::new(style.stroke.width, style.stroke.color.gamma_multiply(0.45))
            };
            painter.rect_stroke(rect, CornerRadius::same(RADIUS), st, StrokeKind::Inside);
        }

        let start_x = rect.center().x - content_w * 0.5;
        let mut text_x = start_x;
        if let Some(ic) = icon {
            let ic_rect = Rect::from_min_size(
                pos2(start_x, rect.center().y - style.icon_size * 0.5),
                vec2(style.icon_size, style.icon_size),
            );
            icons::paint(painter, ic_rect, ic, fg);
            text_x += style.icon_size + icon_gap;
        }
        if !label.is_empty() {
            painter.text(
                pos2(text_x, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(style.font_size),
                fg,
            );
        }
    }

    if enabled {
        resp.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        resp
    }
}

pub fn primary_button(ui: &mut Ui, p: &Palette, icon: Option<Icon>, label: &str, enabled: bool) -> Response {
    styled_button(
        ui,
        label,
        icon,
        ButtonStyle {
            bg: p.accent,
            bg_hover: p.accent_hover,
            text_color: Color32::WHITE,
            stroke: Stroke::NONE,
            padding: vec2(16.0, 9.0),
            font_size: 13.0,
            icon_size: 15.0,
        },
        enabled,
    )
}

pub fn danger_button(ui: &mut Ui, p: &Palette, icon: Option<Icon>, label: &str, enabled: bool) -> Response {
    styled_button(
        ui,
        label,
        icon,
        ButtonStyle {
            bg: p.danger,
            bg_hover: p.danger_hover,
            text_color: Color32::WHITE,
            stroke: Stroke::NONE,
            padding: vec2(16.0, 9.0),
            font_size: 13.0,
            icon_size: 15.0,
        },
        enabled,
    )
}

pub fn outline_button(ui: &mut Ui, p: &Palette, icon: Option<Icon>, label: &str, enabled: bool) -> Response {
    styled_button(
        ui,
        label,
        icon,
        ButtonStyle {
            bg: p.bg_2,
            bg_hover: p.bg_hover,
            text_color: p.text_0,
            stroke: Stroke::new(1.0_f32, p.border),
            padding: vec2(15.0, 9.0),
            font_size: 13.0,
            icon_size: 15.0,
        },
        enabled,
    )
}

pub fn small_button(ui: &mut Ui, p: &Palette, icon: Option<Icon>, label: &str, enabled: bool) -> Response {
    styled_button(
        ui,
        label,
        icon,
        ButtonStyle {
            bg: p.bg_3,
            bg_hover: p.bg_hover,
            text_color: p.text_0,
            stroke: Stroke::new(1.0_f32, p.border),
            padding: vec2(11.0, 6.0),
            font_size: 12.0,
            icon_size: 14.0,
        },
        enabled,
    )
}

pub fn chip_button(ui: &mut Ui, p: &Palette, icon: Icon, label: &str, active: bool) -> Response {
    styled_button(
        ui,
        label,
        Some(icon),
        ButtonStyle {
            bg: if active { p.accent_dim } else { p.bg_3 },
            bg_hover: if active { p.accent_dim } else { p.bg_hover },
            text_color: if active { p.accent_text } else { p.text_1 },
            stroke: Stroke::new(1.0_f32, if active { p.accent_border } else { p.border }),
            padding: vec2(10.0, 5.0),
            font_size: 11.5,
            icon_size: 13.0,
        },
        true,
    )
}

pub fn render_inline_rich_text(ui: &mut Ui, p: &Palette, text: &str) {
    let mut job = LayoutJob::default();
    let normal = TextFormat {
        font_id: FontId::proportional(13.5),
        color: p.text_1,
        ..Default::default()
    };
    let bold = TextFormat {
        font_id: FontId::proportional(13.5),
        color: p.text_0,
        ..Default::default()
    };
    let code = TextFormat {
        font_id: FontId::monospace(12.0),
        color: p.accent_text,
        background: p.bg_3,
        ..Default::default()
    };

    let mut rest = text;
    while !rest.is_empty() {
        let next_b = rest.find("<b>");
        let next_c = rest.find("<code>");
        match (next_b, next_c) {
            (Some(b), Some(c)) if b < c => {
                if b > 0 {
                    job.append(&rest[..b], 0.0, normal.clone());
                }
                rest = &rest[b + 3..];
                if let Some(end) = rest.find("</b>") {
                    job.append(&rest[..end], 0.0, bold.clone());
                    rest = &rest[end + 4..];
                } else {
                    job.append(rest, 0.0, bold.clone());
                    break;
                }
            }
            (_, Some(c)) => {
                if c > 0 {
                    job.append(&rest[..c], 0.0, normal.clone());
                }
                rest = &rest[c + 6..];
                if let Some(end) = rest.find("</code>") {
                    job.append(&format!(" {} ", &rest[..end]), 0.0, code.clone());
                    rest = &rest[end + 7..];
                } else {
                    job.append(rest, 0.0, code.clone());
                    break;
                }
            }
            (Some(b), None) => {
                if b > 0 {
                    job.append(&rest[..b], 0.0, normal.clone());
                }
                rest = &rest[b + 3..];
                if let Some(end) = rest.find("</b>") {
                    job.append(&rest[..end], 0.0, bold.clone());
                    rest = &rest[end + 4..];
                } else {
                    job.append(rest, 0.0, bold.clone());
                    break;
                }
            }
            (None, None) => {
                job.append(rest, 0.0, normal.clone());
                break;
            }
        }
    }

    ui.label(job);
}

pub fn format_number(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(b as char);
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    Unchecked,
    Indeterminate,
    Checked,
}

pub fn paint_tristate_checkbox(
    painter: &egui::Painter,
    rect: Rect,
    state: TriState,
    p: &Palette,
    alpha: f32,
) {
    let active = state != TriState::Unchecked;
    let bg = if active {
        p.accent.gamma_multiply(alpha)
    } else {
        p.bg_3.gamma_multiply(alpha)
    };
    let border = if active {
        p.accent.gamma_multiply(alpha)
    } else {
        p.border.gamma_multiply(alpha)
    };
    painter.rect_filled(rect, CornerRadius::same(4), bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0_f32, border),
        StrokeKind::Inside,
    );

    match state {
        TriState::Checked => {
            let p1 = pos2(rect.left() + 3.0, rect.center().y);
            let p2 = pos2(rect.center().x - 0.5, rect.bottom() - 3.5);
            let p3 = pos2(rect.right() - 2.5, rect.top() + 3.5);
            painter.add(egui::Shape::line(
                vec![p1, p2, p3],
                Stroke::new(1.8_f32, Color32::WHITE),
            ));
        }
        TriState::Indeterminate => {
            painter.line_segment(
                [
                    pos2(rect.left() + 3.5, rect.center().y),
                    pos2(rect.right() - 3.5, rect.center().y),
                ],
                Stroke::new(1.8_f32, Color32::WHITE),
            );
        }
        TriState::Unchecked => {}
    }
}

pub fn paint_dashed_rect(
    painter: &egui::Painter,
    rect: Rect,
    stroke: Stroke,
    dash: f32,
    gap: f32,
) {
    let step = dash + gap;
    let mut x = rect.left();
    while x < rect.right() {
        let x2 = (x + dash).min(rect.right());
        painter.line_segment([pos2(x, rect.top()), pos2(x2, rect.top())], stroke);
        painter.line_segment([pos2(x, rect.bottom()), pos2(x2, rect.bottom())], stroke);
        x += step;
    }
    let mut y = rect.top();
    while y < rect.bottom() {
        let y2 = (y + dash).min(rect.bottom());
        painter.line_segment([pos2(rect.left(), y), pos2(rect.left(), y2)], stroke);
        painter.line_segment([pos2(rect.right(), y), pos2(rect.right(), y2)], stroke);
        y += step;
    }
}
