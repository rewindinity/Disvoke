use crate::app::Page;
use egui::{
    Align2, Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame,
    Margin, Pos2, Rect, Response, Sense, Stroke, StrokeKind, TextFormat, TextStyle, Ui, Vec2,
    Visuals, pos2, text::LayoutJob, vec2,
};
use std::sync::Arc;

pub const BG_0: Color32 = Color32::from_rgb(0x0d, 0x0e, 0x11);
pub const BG_1: Color32 = Color32::from_rgb(0x13, 0x15, 0x1a);
pub const BG_2: Color32 = Color32::from_rgb(0x1a, 0x1d, 0x24);
pub const BG_3: Color32 = Color32::from_rgb(0x22, 0x26, 0x2f);
pub const BG_HOVER: Color32 = Color32::from_rgb(0x2a, 0x2f, 0x3a);
pub const BORDER: Color32 = Color32::from_rgb(0x2e, 0x33, 0x40);
pub const ACCENT: Color32 = Color32::from_rgb(0x58, 0x65, 0xf2);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x47, 0x52, 0xc4);
pub const ACCENT_DIM: Color32 = Color32::from_rgba_premultiplied(13, 15, 36, 38);
pub const DANGER: Color32 = Color32::from_rgb(0xed, 0x42, 0x45);
pub const DANGER_HOVER: Color32 = Color32::from_rgb(0xc0, 0x35, 0x37);
pub const SUCCESS: Color32 = Color32::from_rgb(0x57, 0xf2, 0x87);
pub const SUCCESS_DIM: Color32 = Color32::from_rgba_premultiplied(4, 12, 7, 13);
pub const WARN: Color32 = Color32::from_rgb(0xfe, 0xe7, 0x5c);
pub const TEXT_0: Color32 = Color32::from_rgb(0xf2, 0xf3, 0xf5);
pub const TEXT_1: Color32 = Color32::from_rgb(0xb5, 0xba, 0xc1);
pub const TEXT_2: Color32 = Color32::from_rgb(0x6d, 0x72, 0x80);

pub const RADIUS: u8 = 8;
pub const RADIUS_LG: u8 = 12;

pub fn configure_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    let fallback_paths = [
        "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
        "C:\\Windows\\Fonts\\seguisym.ttf",
        "/System/Library/Fonts/Apple Symbols.ttf",
    ];

    for (idx, path) in fallback_paths.iter().enumerate() {
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

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG_0;
    visuals.window_fill = BG_2;
    visuals.faint_bg_color = BG_1;
    visuals.extreme_bg_color = BG_3;
    visuals.code_bg_color = BG_2;
    visuals.override_text_color = Some(TEXT_0);
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = WARN;
    visuals.error_fg_color = DANGER;
    visuals.window_corner_radius = CornerRadius::same(RADIUS_LG);
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.menu_corner_radius = CornerRadius::same(RADIUS);

    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(88, 101, 242, 90);
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);

    visuals.widgets.noninteractive.bg_fill = BG_2;
    visuals.widgets.noninteractive.weak_bg_fill = BG_2;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_1);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS);

    visuals.widgets.inactive.bg_fill = BG_3;
    visuals.widgets.inactive.weak_bg_fill = BG_3;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_0);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(RADIUS);

    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BG_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT_0);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(RADIUS);

    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.weak_bg_fill = ACCENT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
    visuals.widgets.active.corner_radius = CornerRadius::same(RADIUS);

    visuals.widgets.open.bg_fill = BG_3;
    visuals.widgets.open.weak_bg_fill = BG_3;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, TEXT_0);
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

pub fn card_frame() -> Frame {
    Frame::new()
        .fill(BG_2)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(RADIUS_LG))
        .inner_margin(Margin::symmetric(24, 20))
}

pub fn panel_box_frame() -> Frame {
    Frame::new()
        .fill(BG_1)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(RADIUS_LG))
        .inner_margin(Margin::same(10))
}

pub fn page_title(ui: &mut Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(22.0)
            .strong()
            .color(TEXT_0),
    );
}

pub fn section_title(ui: &mut Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(14.0)
            .strong()
            .color(TEXT_0),
    );
    ui.add_space(8.0);
}

pub fn field_label(ui: &mut Ui, text: &str) {
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .size(11.0)
            .strong()
            .color(TEXT_1),
    );
}

pub fn field_hint(ui: &mut Ui, text: &str) {
    if !text.is_empty() {
        ui.label(egui::RichText::new(text).size(11.5).color(TEXT_2));
    }
}

struct ButtonStyle {
    bg: Color32,
    bg_hover: Color32,
    text_color: Color32,
    stroke: Stroke,
    padding: Vec2,
    font_size: f32,
}

fn styled_button(ui: &mut Ui, label: &str, style: ButtonStyle, enabled: bool) -> Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::proportional(style.font_size),
        style.text_color,
    );
    let desired = galley.size() + style.padding * 2.0;
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
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
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(style.font_size),
            fg,
        );
    }

    if enabled {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    }
}

pub fn primary_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    styled_button(
        ui,
        label,
        ButtonStyle {
            bg: ACCENT,
            bg_hover: ACCENT_HOVER,
            text_color: Color32::WHITE,
            stroke: Stroke::NONE,
            padding: vec2(18.0, 9.0),
            font_size: 13.0,
        },
        enabled,
    )
}

pub fn danger_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    styled_button(
        ui,
        label,
        ButtonStyle {
            bg: DANGER,
            bg_hover: DANGER_HOVER,
            text_color: Color32::WHITE,
            stroke: Stroke::NONE,
            padding: vec2(18.0, 9.0),
            font_size: 13.0,
        },
        enabled,
    )
}

pub fn outline_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    styled_button(
        ui,
        label,
        ButtonStyle {
            bg: Color32::TRANSPARENT,
            bg_hover: BG_2,
            text_color: TEXT_0,
            stroke: Stroke::new(1.0_f32, BORDER),
            padding: vec2(18.0, 9.0),
            font_size: 13.0,
        },
        enabled,
    )
}

pub fn small_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    styled_button(
        ui,
        label,
        ButtonStyle {
            bg: BG_3,
            bg_hover: BG_HOVER,
            text_color: TEXT_0,
            stroke: Stroke::new(1.0_f32, BORDER),
            padding: vec2(12.0, 6.0),
            font_size: 12.0,
        },
        enabled,
    )
}

pub fn render_inline_rich_text(ui: &mut Ui, text: &str) {
    let mut job = LayoutJob::default();
    let normal = TextFormat {
        font_id: FontId::proportional(13.5),
        color: TEXT_1,
        ..Default::default()
    };
    let bold = TextFormat {
        font_id: FontId::proportional(13.5),
        color: TEXT_0,
        ..Default::default()
    };
    let code = TextFormat {
        font_id: FontId::monospace(12.0),
        color: TEXT_0,
        background: BG_2,
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

pub fn paint_nav_icon(painter: &egui::Painter, rect: Rect, page: Page, color: Color32) {
    let stroke = Stroke::new(1.75_f32, color);
    let s = rect.width().min(rect.height()) / 24.0;
    let p = |x: f32, y: f32| pos2(rect.left() + x * s, rect.top() + y * s);
    let r = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(p(x, y), vec2(w * s, h * s));

    match page {
        Page::Dashboard => {
            for (x, y) in [(3.0, 3.0), (14.0, 3.0), (3.0, 14.0), (14.0, 14.0)] {
                painter.rect_stroke(r(x, y, 7.0, 7.0), CornerRadius::same(1), stroke, StrokeKind::Middle);
            }
        }
        Page::Pack => {
            let pts = vec![
                p(2.5, 5.0),
                p(9.0, 5.0),
                p(11.0, 8.0),
                p(21.5, 8.0),
                p(21.5, 19.5),
                p(2.5, 19.5),
            ];
            painter.add(egui::Shape::closed_line(pts, stroke));
        }
        Page::Review => {
            let box_pts = vec![p(20.0, 13.0), p(20.0, 20.0), p(4.0, 20.0), p(4.0, 4.0), p(15.0, 4.0)];
            painter.add(egui::Shape::line(box_pts, stroke));
            let check = vec![p(9.0, 11.5), p(12.0, 14.5), p(21.5, 5.0)];
            painter.add(egui::Shape::line(check, stroke));
        }
        Page::Progress => {
            let pts = vec![
                p(2.0, 12.0),
                p(6.0, 12.0),
                p(9.0, 4.0),
                p(15.0, 20.0),
                p(18.0, 12.0),
                p(22.0, 12.0),
            ];
            painter.add(egui::Shape::line(pts, stroke));
        }
        Page::Settings => {
            let center = p(12.0, 12.0);
            painter.circle_stroke(center, 3.2 * s, stroke);
            painter.circle_stroke(center, 7.2 * s, stroke);
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::FRAC_PI_4;
                let (sin, cos) = angle.sin_cos();
                let a = pos2(center.x + cos * 7.2 * s, center.y + sin * 7.2 * s);
                let b = pos2(center.x + cos * 9.8 * s, center.y + sin * 9.8 * s);
                painter.line_segment([a, b], Stroke::new(2.0_f32, color));
            }
        }
    }
}

pub fn paint_folder_large(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(2.0_f32, color);
    let s = rect.width().min(rect.height()) / 24.0;
    let p = |x: f32, y: f32| pos2(rect.left() + x * s, rect.top() + y * s);
    let pts = vec![
        p(2.5, 5.0),
        p(9.0, 5.0),
        p(11.0, 8.0),
        p(21.5, 8.0),
        p(21.5, 19.5),
        p(2.5, 19.5),
    ];
    painter.add(egui::Shape::closed_line(pts, stroke));
}

pub fn paint_chevron(painter: &egui::Painter, center: Pos2, expanded: bool, color: Color32) {
    let pts = if expanded {
        vec![
            pos2(center.x - 4.0, center.y - 2.0),
            pos2(center.x + 4.0, center.y - 2.0),
            pos2(center.x, center.y + 3.0),
        ]
    } else {
        vec![
            pos2(center.x - 2.0, center.y - 4.0),
            pos2(center.x + 3.0, center.y),
            pos2(center.x - 2.0, center.y + 4.0),
        ]
    };
    painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
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
