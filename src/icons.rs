use egui::{Color32, CornerRadius, Painter, Rect, Shape, Stroke, StrokeKind, pos2, vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Dashboard,
    Folder,
    FolderOpen,
    Message,
    CheckSquare,
    Activity,
    Settings,
    Trash,
    Stop,
    Refresh,
    Save,
    Search,
    Eye,
    EyeOff,
    Check,
    AlertTriangle,
    Shield,
    Edit,
    History,
    Globe,
    Palette,
    Sun,
    Moon,
    Server,
    ArrowRight,
    Key,
    HelpCircle,
    ChevronDown,
    ChevronRight,
    ExpandAll,
    CollapseAll,
    X,
    Minimize,
    Maximize,
    Restore,
}

pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let dim = rect.width().min(rect.height());
    let s = dim / 24.0;
    let w = (dim * 0.085).clamp(1.3, 2.2);
    let stroke = Stroke::new(w, color);
    let origin = pos2(
        rect.center().x - 12.0 * s,
        rect.center().y - 12.0 * s,
    );
    let p = |x: f32, y: f32| pos2(origin.x + x * s, origin.y + y * s);
    let r = |x: f32, y: f32, rw: f32, rh: f32| Rect::from_min_size(p(x, y), vec2(rw * s, rh * s));

    match icon {
        Icon::Dashboard => {
            for (x, y) in [(3.0, 3.0), (14.0, 3.0), (3.0, 14.0), (14.0, 14.0)] {
                painter.rect_stroke(r(x, y, 7.0, 7.0), CornerRadius::same(2), stroke, StrokeKind::Middle);
            }
        }
        Icon::Folder => {
            let pts = vec![
                p(2.5, 5.5),
                p(9.0, 5.5),
                p(11.0, 8.5),
                p(21.5, 8.5),
                p(21.5, 19.5),
                p(2.5, 19.5),
            ];
            painter.add(Shape::closed_line(pts, stroke));
        }
        Icon::FolderOpen => {
            let top = vec![p(2.5, 18.5), p(2.5, 5.5), p(9.0, 5.5), p(11.0, 8.0), p(19.5, 8.0), p(19.5, 11.0)];
            painter.add(Shape::line(top, stroke));
            let front = vec![p(2.5, 19.0), p(5.5, 11.5), p(22.0, 11.5), p(19.0, 19.0)];
            painter.add(Shape::closed_line(front, stroke));
        }
        Icon::Message => {
            let pts = vec![
                p(3.5, 4.5),
                p(20.5, 4.5),
                p(20.5, 16.5),
                p(9.5, 16.5),
                p(4.5, 20.5),
                p(5.5, 16.5),
                p(3.5, 16.5),
            ];
            painter.add(Shape::closed_line(pts, stroke));
            painter.line_segment([p(7.5, 9.0), p(16.5, 9.0)], stroke);
            painter.line_segment([p(7.5, 12.5), p(13.5, 12.5)], stroke);
        }
        Icon::CheckSquare => {
            let box_pts = vec![
                p(20.0, 12.5),
                p(20.0, 20.0),
                p(4.0, 20.0),
                p(4.0, 4.0),
                p(15.5, 4.0),
            ];
            painter.add(Shape::line(box_pts, stroke));
            painter.add(Shape::line(vec![p(8.5, 11.5), p(12.0, 15.0), p(21.5, 5.0)], stroke));
        }
        Icon::Activity => {
            let pts = vec![
                p(2.0, 12.0),
                p(6.0, 12.0),
                p(9.0, 4.0),
                p(15.0, 20.0),
                p(18.0, 12.0),
                p(22.0, 12.0),
            ];
            painter.add(Shape::line(pts, stroke));
        }
        Icon::Settings => {
            let c = p(12.0, 12.0);
            painter.circle_stroke(c, 3.2 * s, stroke);
            painter.circle_stroke(c, 7.0 * s, stroke);
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::FRAC_PI_4;
                let (sin, cos) = angle.sin_cos();
                let a = pos2(c.x + cos * 7.0 * s, c.y + sin * 7.0 * s);
                let b = pos2(c.x + cos * 9.8 * s, c.y + sin * 9.8 * s);
                painter.line_segment([a, b], Stroke::new(w + 0.3, color));
            }
        }
        Icon::Trash => {
            painter.line_segment([p(3.5, 6.5), p(20.5, 6.5)], stroke);
            painter.add(Shape::line(vec![p(9.0, 6.5), p(9.0, 4.0), p(15.0, 4.0), p(15.0, 6.5)], stroke));
            let bin = vec![p(5.5, 6.5), p(6.8, 20.5), p(17.2, 20.5), p(18.5, 6.5)];
            painter.add(Shape::line(bin, stroke));
            painter.line_segment([p(10.0, 10.5), p(10.3, 16.8)], stroke);
            painter.line_segment([p(14.0, 10.5), p(13.7, 16.8)], stroke);
        }
        Icon::Stop => {
            painter.rect_filled(r(6.0, 6.0, 12.0, 12.0), CornerRadius::same(2), color);
        }
        Icon::Refresh => {
            let mut pts = Vec::with_capacity(14);
            for i in 0..=12 {
                let t = -std::f32::consts::FRAC_PI_4 + (i as f32 / 12.0) * std::f32::consts::PI * 1.55;
                let (sin, cos) = t.sin_cos();
                pts.push(p(12.0 + cos * 7.5, 12.0 + sin * 7.5));
            }
            painter.add(Shape::line(pts, stroke));
            painter.add(Shape::line(vec![p(14.0, 4.5), p(18.5, 6.5), p(16.5, 10.5)], stroke));
        }
        Icon::Save => {
            let outer = vec![
                p(4.0, 4.0),
                p(16.5, 4.0),
                p(20.0, 7.5),
                p(20.0, 20.0),
                p(4.0, 20.0),
            ];
            painter.add(Shape::closed_line(outer, stroke));
            painter.rect_stroke(r(7.5, 4.0, 7.0, 5.0), CornerRadius::ZERO, stroke, StrokeKind::Middle);
            painter.rect_stroke(r(7.0, 13.0, 10.0, 7.0), CornerRadius::ZERO, stroke, StrokeKind::Middle);
        }
        Icon::Search => {
            painter.circle_stroke(p(10.5, 10.5), 6.2 * s, stroke);
            painter.line_segment([p(15.2, 15.2), p(20.5, 20.5)], stroke);
        }
        Icon::Eye => {
            painter.add(Shape::line(
                vec![p(2.5, 12.0), p(6.5, 7.2), p(12.0, 6.0), p(17.5, 7.2), p(21.5, 12.0)],
                stroke,
            ));
            painter.add(Shape::line(
                vec![p(2.5, 12.0), p(6.5, 16.8), p(12.0, 18.0), p(17.5, 16.8), p(21.5, 12.0)],
                stroke,
            ));
            painter.circle_stroke(p(12.0, 12.0), 3.0 * s, stroke);
        }
        Icon::EyeOff => {
            painter.add(Shape::line(
                vec![p(2.5, 12.0), p(6.5, 7.2), p(12.0, 6.0), p(17.5, 7.2), p(21.5, 12.0)],
                stroke,
            ));
            painter.add(Shape::line(
                vec![p(2.5, 12.0), p(6.5, 16.8), p(12.0, 18.0), p(17.5, 16.8), p(21.5, 12.0)],
                stroke,
            ));
            painter.line_segment([p(3.5, 3.5), p(20.5, 20.5)], stroke);
        }
        Icon::Check => {
            painter.add(Shape::line(vec![p(4.5, 12.5), p(9.8, 17.8), p(20.0, 6.5)], stroke));
        }
        Icon::AlertTriangle => {
            let tri = vec![p(12.0, 3.5), p(21.5, 19.5), p(2.5, 19.5)];
            painter.add(Shape::closed_line(tri, stroke));
            painter.line_segment([p(12.0, 9.5), p(12.0, 14.2)], stroke);
            painter.circle_filled(p(12.0, 17.0), 1.1 * s, color);
        }
        Icon::Shield => {
            let pts = vec![
                p(12.0, 3.0),
                p(19.5, 6.0),
                p(19.5, 12.0),
                p(16.5, 17.5),
                p(12.0, 21.0),
                p(7.5, 17.5),
                p(4.5, 12.0),
                p(4.5, 6.0),
            ];
            painter.add(Shape::closed_line(pts, stroke));
            painter.add(Shape::line(vec![p(8.8, 12.0), p(11.2, 14.5), p(15.8, 9.5)], stroke));
        }
        Icon::Edit => {
            let body = vec![p(4.0, 20.0), p(5.5, 14.5), p(16.2, 3.8), p(20.2, 7.8), p(9.5, 18.5)];
            painter.add(Shape::closed_line(body, stroke));
            painter.line_segment([p(13.8, 6.2), p(17.8, 10.2)], stroke);
        }
        Icon::History => {
            painter.circle_stroke(p(12.0, 12.0), 8.2 * s, stroke);
            painter.add(Shape::line(vec![p(12.0, 7.0), p(12.0, 12.0), p(15.8, 14.2)], stroke));
        }
        Icon::Globe => {
            let c = p(12.0, 12.0);
            painter.circle_stroke(c, 8.5 * s, stroke);
            painter.line_segment([p(3.5, 12.0), p(20.5, 12.0)], stroke);
            painter.line_segment([p(12.0, 3.5), p(12.0, 20.5)], stroke);
            painter.add(Shape::line(vec![p(5.2, 7.5), p(12.0, 8.8), p(18.8, 7.5)], stroke));
            painter.add(Shape::line(vec![p(5.2, 16.5), p(12.0, 15.2), p(18.8, 16.5)], stroke));
        }
        Icon::Palette => {
            painter.circle_stroke(p(12.0, 12.0), 8.5 * s, stroke);
            for (x, y) in [(8.5, 9.0), (12.5, 7.5), (16.0, 10.5), (15.0, 15.0)] {
                painter.circle_filled(p(x, y), 1.4 * s, color);
            }
        }
        Icon::Sun => {
            let c = p(12.0, 12.0);
            painter.circle_stroke(c, 4.2 * s, stroke);
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::FRAC_PI_4;
                let (sin, cos) = angle.sin_cos();
                let a = pos2(c.x + cos * 6.8 * s, c.y + sin * 6.8 * s);
                let b = pos2(c.x + cos * 9.5 * s, c.y + sin * 9.5 * s);
                painter.line_segment([a, b], stroke);
            }
        }
        Icon::Moon => {
            let pts = vec![
                p(18.5, 14.8),
                p(16.0, 18.2),
                p(12.0, 19.6),
                p(7.6, 18.2),
                p(5.0, 14.5),
                p(5.0, 9.8),
                p(7.6, 6.0),
                p(11.8, 4.6),
                p(10.0, 8.2),
                p(10.2, 12.2),
                p(13.0, 14.8),
            ];
            painter.add(Shape::closed_line(pts, stroke));
        }
        Icon::Server => {
            painter.rect_stroke(r(3.5, 4.0, 17.0, 6.5), CornerRadius::same(2), stroke, StrokeKind::Middle);
            painter.rect_stroke(r(3.5, 13.5, 17.0, 6.5), CornerRadius::same(2), stroke, StrokeKind::Middle);
            painter.circle_filled(p(7.2, 7.2), 1.1 * s, color);
            painter.circle_filled(p(7.2, 16.8), 1.1 * s, color);
            painter.line_segment([p(11.0, 7.2), p(17.5, 7.2)], stroke);
            painter.line_segment([p(11.0, 16.8), p(17.5, 16.8)], stroke);
        }
        Icon::ArrowRight => {
            painter.line_segment([p(4.5, 12.0), p(19.0, 12.0)], stroke);
            painter.add(Shape::line(vec![p(13.5, 6.5), p(19.5, 12.0), p(13.5, 17.5)], stroke));
        }
        Icon::Key => {
            painter.circle_stroke(p(8.2, 12.0), 4.2 * s, stroke);
            painter.line_segment([p(12.4, 12.0), p(21.0, 12.0)], stroke);
            painter.line_segment([p(17.5, 12.0), p(17.5, 15.8)], stroke);
            painter.line_segment([p(20.5, 12.0), p(20.5, 15.0)], stroke);
        }
        Icon::HelpCircle => {
            painter.circle_stroke(p(12.0, 12.0), 8.5 * s, stroke);
            painter.add(Shape::line(
                vec![p(9.5, 9.2), p(10.5, 7.5), p(12.5, 7.2), p(14.5, 8.5), p(14.2, 10.8), p(12.0, 12.2), p(12.0, 14.0)],
                stroke,
            ));
            painter.circle_filled(p(12.0, 17.0), 1.1 * s, color);
        }
        Icon::ChevronDown => {
            painter.add(Shape::line(vec![p(6.5, 9.5), p(12.0, 15.0), p(17.5, 9.5)], stroke));
        }
        Icon::ChevronRight => {
            painter.add(Shape::line(vec![p(9.5, 6.5), p(15.0, 12.0), p(9.5, 17.5)], stroke));
        }
        Icon::ExpandAll => {
            painter.add(Shape::line(vec![p(7.0, 8.5), p(12.0, 4.5), p(17.0, 8.5)], stroke));
            painter.add(Shape::line(vec![p(7.0, 15.5), p(12.0, 19.5), p(17.0, 15.5)], stroke));
        }
        Icon::CollapseAll => {
            painter.add(Shape::line(vec![p(7.0, 5.0), p(12.0, 9.5), p(17.0, 5.0)], stroke));
            painter.add(Shape::line(vec![p(7.0, 19.0), p(12.0, 14.5), p(17.0, 19.0)], stroke));
        }
        Icon::X => {
            painter.line_segment([p(6.0, 6.0), p(18.0, 18.0)], stroke);
            painter.line_segment([p(18.0, 6.0), p(6.0, 18.0)], stroke);
        }
        Icon::Minimize => {
            painter.line_segment([p(5.5, 12.0), p(18.5, 12.0)], stroke);
        }
        Icon::Maximize => {
            painter.rect_stroke(r(5.5, 5.5, 13.0, 13.0), CornerRadius::same(1), stroke, StrokeKind::Middle);
        }
        Icon::Restore => {
            painter.rect_stroke(r(5.0, 8.0, 11.0, 11.0), CornerRadius::same(1), stroke, StrokeKind::Middle);
            painter.add(Shape::line(
                vec![p(8.5, 8.0), p(8.5, 5.0), p(19.0, 5.0), p(19.0, 15.5), p(16.0, 15.5)],
                stroke,
            ));
        }
    }
}
