use crate::app::{DisvokeApp, TreeRow};
use crate::theme::{
    self, ACCENT, BG_2, BG_3, BORDER, RADIUS, TEXT_0, TEXT_1, TEXT_2, format_number,
};
use egui::{
    Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Layout, Rect, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui) {
    let title = app.i18n.t("review.title").to_string();
    let select_all_lbl = app.i18n.t("review.selectAll").to_string();
    let deselect_all_lbl = app.i18n.t("review.deselectAll").to_string();
    let selected_lbl = app.i18n.t("review.selected").to_string();
    let selected_count = app.selected_ids.len();

    ui.horizontal(|ui| {
        theme::page_title(ui, &title);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{} {selected_lbl}", format_number(selected_count)))
                    .size(13.0)
                    .color(TEXT_2),
            );
            ui.add_space(6.0);
            if theme::small_button(ui, &deselect_all_lbl, !app.filtered_messages.is_empty()).clicked() {
                app.deselect_all();
            }
            if theme::small_button(ui, &select_all_lbl, !app.filtered_messages.is_empty()).clicked() {
                app.select_all_filtered();
            }
        });
    });

    ui.add_space(16.0);

    let footer_h = 56.0;
    let tree_h = (ui.available_height() - footer_h).max(160.0);

    let server_prefix = app.i18n.t("review.serverGroup").to_string();
    let dm_group_label = app.i18n.t("review.dmGroup").to_string();
    let messages_suffix = app.i18n.t("review.messages").to_string();
    let no_content_lbl = app.i18n.t("review.noContent").to_string();
    let empty_lbl = app.i18n.t("review.empty").to_string();
    let no_messages_lbl = app.i18n.t("review.noMessages").to_string();

    let mut toggle_group: Option<usize> = None;
    let mut toggle_channel: Option<(usize, usize)> = None;
    let mut toggle_msg: Option<String> = None;

    theme::panel_box_frame().show(ui, |ui| {
        ui.set_min_size(vec2(ui.available_width(), tree_h));
        ui.set_max_height(tree_h);

        if app.pack_data.is_none()
            && app.dm.fetched_info.is_none()
            && app.filtered_messages.is_empty()
        {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(empty_lbl).color(TEXT_2));
            });
            return;
        }
        if app.filtered_messages.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(no_messages_lbl).color(TEXT_2));
            });
            return;
        }

        let row_h = 30.0;
        let total_rows = app.visible_rows.len();

        ScrollArea::vertical()
            .id_salt("review_tree_scroll")
            .auto_shrink([false, false])
            .show_rows(ui, row_h, total_rows, |ui, range| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for row_idx in range {
                    let row = app.visible_rows[row_idx];
                    let avail_w = ui.available_width();
                    match row {
                        TreeRow::Group(g_idx) => {
                            let grp = &app.review_groups[g_idx];
                            let (rect, resp) =
                                ui.allocate_exact_size(vec2(avail_w, row_h), Sense::click());
                            if ui.is_rect_visible(rect) {
                                let fill = if resp.hovered() { BG_3 } else { BG_2 };
                                let painter = ui.painter();
                                painter.rect_filled(rect, CornerRadius::same(RADIUS), fill);

                                let label = if grp.is_dm_group {
                                    dm_group_label.clone()
                                } else {
                                    format!("{server_prefix} {}", grp.name)
                                };
                                let count_txt = format!(
                                    "{} {messages_suffix}",
                                    format_number(grp.total_messages)
                                );

                                let label_galley = painter.layout_no_wrap(
                                    label,
                                    FontId::proportional(13.0),
                                    TEXT_0,
                                );
                                let label_pos = pos2(
                                    rect.left() + 12.0,
                                    rect.center().y - label_galley.size().y / 2.0,
                                );
                                let label_w = label_galley.size().x;
                                painter.galley(label_pos, label_galley, TEXT_0);

                                painter.text(
                                    pos2(rect.left() + 20.0 + label_w, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    count_txt,
                                    FontId::proportional(11.0),
                                    TEXT_2,
                                );

                                theme::paint_chevron(
                                    painter,
                                    pos2(rect.right() - 16.0, rect.center().y),
                                    grp.expanded,
                                    TEXT_1,
                                );
                            }
                            if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                                toggle_group = Some(g_idx);
                            }
                        }
                        TreeRow::Channel(g_idx, c_idx) => {
                            let ch = &app.review_groups[g_idx].channels[c_idx];
                            let (full_rect, resp) =
                                ui.allocate_exact_size(vec2(avail_w, row_h), Sense::click());
                            let rect = Rect::from_min_max(
                                pos2(full_rect.left() + 20.0, full_rect.top()),
                                full_rect.max,
                            );
                            if ui.is_rect_visible(rect) {
                                let hovered = resp.hovered();
                                let painter = ui.painter();
                                if hovered {
                                    painter.rect_filled(rect, CornerRadius::same(RADIUS), BG_2);
                                }
                                let prefix = if ch.is_dm { "@" } else { "#" };
                                let title = format!("{prefix} {}", ch.name);
                                let fg = if hovered { TEXT_0 } else { TEXT_1 };
                                painter.text(
                                    pos2(rect.left() + 12.0, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    title,
                                    FontId::proportional(12.5),
                                    fg,
                                );

                                let badge_txt = format_number(ch.message_indices.len());
                                let badge_galley = painter.layout_no_wrap(
                                    badge_txt,
                                    FontId::proportional(11.0),
                                    TEXT_2,
                                );
                                let badge_w = badge_galley.size().x + 14.0;
                                let badge_rect = Rect::from_center_size(
                                    pos2(rect.right() - badge_w / 2.0 - 10.0, rect.center().y),
                                    vec2(badge_w, 18.0),
                                );
                                painter.rect_filled(badge_rect, CornerRadius::same(9), BG_3);
                                painter.galley(
                                    pos2(
                                        badge_rect.center().x - badge_galley.size().x / 2.0,
                                        badge_rect.center().y - badge_galley.size().y / 2.0,
                                    ),
                                    badge_galley,
                                    TEXT_2,
                                );
                            }
                            if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                                toggle_channel = Some((g_idx, c_idx));
                            }
                        }
                        TreeRow::Message(msg_idx) => {
                            let item = &app.filtered_messages[msg_idx];
                            let msg = &item.msg;
                            let already_deleted = app.history.deleted_ids.contains(&msg.id);
                            let checked = !already_deleted && app.selected_ids.contains(&msg.id);

                            let sense = if already_deleted {
                                Sense::hover()
                            } else {
                                Sense::click()
                            };
                            let (full_rect, resp) =
                                ui.allocate_exact_size(vec2(avail_w, row_h), sense);
                            let rect = Rect::from_min_max(
                                pos2(full_rect.left() + 42.0, full_rect.top()),
                                full_rect.max,
                            );

                            if ui.is_rect_visible(rect) {
                                let painter = ui.painter();
                                if !already_deleted && resp.hovered() {
                                    painter.rect_filled(rect, CornerRadius::same(6), BG_3);
                                }

                                let alpha = if already_deleted { 0.4 } else { 1.0 };
                                let cb_rect = Rect::from_center_size(
                                    pos2(rect.left() + 14.0, rect.center().y),
                                    vec2(14.0, 14.0),
                                );
                                let cb_bg = if checked {
                                    ACCENT
                                } else {
                                    BG_3.gamma_multiply(alpha)
                                };
                                let cb_border = if checked {
                                    ACCENT
                                } else {
                                    BORDER.gamma_multiply(alpha)
                                };
                                painter.rect_filled(cb_rect, CornerRadius::same(3), cb_bg);
                                painter.rect_stroke(
                                    cb_rect,
                                    CornerRadius::same(3),
                                    Stroke::new(1.0_f32, cb_border),
                                    StrokeKind::Inside,
                                );
                                if checked {
                                    let p1 = pos2(cb_rect.left() + 3.0, cb_rect.center().y);
                                    let p2 = pos2(cb_rect.center().x - 0.5, cb_rect.bottom() - 3.5);
                                    let p3 = pos2(cb_rect.right() - 2.5, cb_rect.top() + 3.5);
                                    painter.add(egui::Shape::line(
                                        vec![p1, p2, p3],
                                        Stroke::new(1.8_f32, Color32::WHITE),
                                    ));
                                }

                                let time_x = cb_rect.right() + 10.0;
                                let time_color = TEXT_2.gamma_multiply(alpha);
                                let time_galley = painter.layout_no_wrap(
                                    msg.formatted_time.clone(),
                                    FontId::proportional(11.0),
                                    time_color,
                                );
                                let time_w = time_galley.size().x;
                                painter.galley(
                                    pos2(time_x, rect.center().y - time_galley.size().y / 2.0),
                                    time_galley,
                                    time_color,
                                );

                                let content_x = time_x + time_w + 12.0;
                                let preview = if msg.preview.is_empty() {
                                    no_content_lbl.as_str()
                                } else {
                                    msg.preview.as_str()
                                };
                                let clip_rect = Rect::from_min_max(
                                    pos2(content_x, rect.top()),
                                    pos2(rect.right() - 8.0, rect.bottom()),
                                );
                                let clipped = painter.with_clip_rect(clip_rect);
                                clipped.text(
                                    pos2(content_x, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    preview,
                                    FontId::proportional(12.0),
                                    TEXT_1.gamma_multiply(alpha),
                                );
                            }

                            if already_deleted {
                                resp.on_hover_text("Already deleted");
                            } else if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                                toggle_msg = Some(msg.id.clone());
                            }
                        }
                    }
                }
            });
    });

    if let Some(g_idx) = toggle_group {
        app.review_groups[g_idx].expanded = !app.review_groups[g_idx].expanded;
        app.rebuild_visible_rows();
    }
    if let Some((g_idx, c_idx)) = toggle_channel {
        let ch = &mut app.review_groups[g_idx].channels[c_idx];
        ch.expanded = !ch.expanded;
        app.rebuild_visible_rows();
    }
    if let Some(id) = toggle_msg
        && !app.selected_ids.remove(&id)
    {
        app.selected_ids.insert(id);
    }

    ui.add_space(16.0);
    let delete_label = app.i18n.t("review.deleteBtn").to_string();
    let can_delete = !app.selected_ids.is_empty() && !app.progress.running;
    if theme::danger_button(ui, &delete_label, can_delete).clicked() {
        app.request_start_deletion();
    }
}
