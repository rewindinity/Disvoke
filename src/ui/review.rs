use crate::app::{DisvokeApp, TreeRow};
use crate::icons::{self, Icon};
use crate::theme::{self, Palette, RADIUS, TriState, format_number};
use crate::ui::dm::render_active_safety_chips;
use egui::{
    Align, Align2, CornerRadius, CursorIcon, FontId, Layout, Rect, RichText, ScrollArea, Sense,
    Ui, pos2, vec2,
};

pub fn render(app: &mut DisvokeApp, ui: &mut Ui, p: &Palette) {
    let title = app.i18n.t("review.title").to_string();
    let subtitle = app.i18n.t("review.subtitle").to_string();
    let select_all_lbl = app.i18n.t("review.selectAll").to_string();
    let deselect_all_lbl = app.i18n.t("review.deselectAll").to_string();
    let expand_all_lbl = app.i18n.t("review.expandAll").to_string();
    let collapse_all_lbl = app.i18n.t("review.collapseAll").to_string();
    let selected_lbl = app.i18n.t("review.selected").to_string();
    let selected_count = app.selected_ids.len();
    let total_filtered = app.filtered_messages.len();
    let has_msgs = total_filtered > 0;

    theme::page_header(ui, p, Icon::CheckSquare, &title, Some(&subtitle));
    ui.add_space(14.0);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
        if theme::small_button(ui, p, Some(Icon::CheckSquare), &select_all_lbl, has_msgs).clicked() {
            app.select_all_filtered();
        }
        if theme::small_button(ui, p, Some(Icon::X), &deselect_all_lbl, selected_count > 0).clicked() {
            app.deselect_all();
        }
        if theme::small_button(ui, p, Some(Icon::ExpandAll), &expand_all_lbl, has_msgs).clicked() {
            app.set_all_expanded(true);
        }
        if theme::small_button(ui, p, Some(Icon::CollapseAll), &collapse_all_lbl, has_msgs).clicked() {
            app.set_all_expanded(false);
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let count_text = format!(
                "{} / {} {selected_lbl}",
                format_number(selected_count),
                format_number(total_filtered)
            );
            theme::pill_badge(
                ui,
                &count_text,
                if selected_count > 0 { p.accent_dim } else { p.bg_2 },
                if selected_count > 0 { p.accent_text } else { p.text_2 },
                Some(if selected_count > 0 { p.accent_border } else { p.border }),
            );
        });
    });

    ui.add_space(12.0);

    let footer_h = 62.0;
    let tree_h = (ui.available_height() - footer_h).max(160.0);

    let dm_group_label = app.i18n.t("review.dmGroup").to_string();
    let messages_suffix = app.i18n.t("review.messages").to_string();
    let no_content_lbl = app.i18n.t("review.noContent").to_string();
    let empty_lbl = app.i18n.t("review.empty").to_string();
    let no_messages_lbl = app.i18n.t("review.noMessages").to_string();
    let already_deleted_tip = app.i18n.t("review.alreadyDeleted").to_string();

    let mut toggle_group_expand: Option<usize> = None;
    let mut toggle_group_check: Option<usize> = None;
    let mut toggle_channel_expand: Option<(usize, usize)> = None;
    let mut toggle_channel_check: Option<(usize, usize)> = None;
    let mut toggle_msg: Option<String> = None;

    theme::inset_frame(p).show(ui, |ui| {
        ui.set_min_size(vec2(ui.available_width(), tree_h));
        ui.set_max_height(tree_h);

        if app.pack_data.is_none() && app.dm.fetched_info.is_none() && !has_msgs {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(empty_lbl).color(p.text_2));
            });
            return;
        }
        if !has_msgs {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(no_messages_lbl).color(p.text_2));
            });
            return;
        }

        let row_h = 32.0;
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
                            let check_state = app.group_selection_state(g_idx);
                            let (rect, resp) =
                                ui.allocate_exact_size(vec2(avail_w, row_h), Sense::click());
                            let cb_hit = Rect::from_min_size(rect.min, vec2(36.0, row_h));

                            if ui.is_rect_visible(rect) {
                                let fill = if resp.hovered() { p.bg_hover } else { p.bg_2 };
                                let painter = ui.painter();
                                painter.rect_filled(rect, CornerRadius::same(RADIUS), fill);

                                let cb_rect = Rect::from_center_size(
                                    pos2(rect.left() + 18.0, rect.center().y),
                                    vec2(15.0, 15.0),
                                );
                                theme::paint_tristate_checkbox(painter, cb_rect, check_state, p, 1.0);

                                let icon_rect = Rect::from_min_size(
                                    pos2(cb_rect.right() + 10.0, rect.center().y - 7.5),
                                    vec2(15.0, 15.0),
                                );
                                let grp_icon = if grp.is_dm_group { Icon::Message } else { Icon::Server };
                                icons::paint(painter, icon_rect, grp_icon, p.accent_text);

                                let label = if grp.is_dm_group {
                                    dm_group_label.clone()
                                } else {
                                    grp.name.clone()
                                };
                                let label_galley = painter.layout_no_wrap(
                                    label,
                                    FontId::proportional(13.0),
                                    p.text_0,
                                );
                                let label_x = icon_rect.right() + 8.0;
                                let label_w = label_galley.size().x;
                                painter.galley(
                                    pos2(label_x, rect.center().y - label_galley.size().y * 0.5),
                                    label_galley,
                                    p.text_0,
                                );

                                let count_txt = format!(
                                    "{} {messages_suffix}",
                                    format_number(grp.total_messages)
                                );
                                painter.text(
                                    pos2(label_x + label_w + 10.0, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    count_txt,
                                    FontId::proportional(11.0),
                                    p.text_2,
                                );

                                let chev = Rect::from_center_size(
                                    pos2(rect.right() - 16.0, rect.center().y),
                                    vec2(14.0, 14.0),
                                );
                                icons::paint(
                                    painter,
                                    chev,
                                    if grp.expanded { Icon::ChevronDown } else { Icon::ChevronRight },
                                    p.text_1,
                                );
                            }

                            let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
                            if resp.clicked() {
                                let clicked_cb = resp
                                    .interact_pointer_pos()
                                    .is_some_and(|pos| cb_hit.contains(pos));
                                if clicked_cb {
                                    toggle_group_check = Some(g_idx);
                                } else {
                                    toggle_group_expand = Some(g_idx);
                                }
                            }
                        }
                        TreeRow::Channel(g_idx, c_idx) => {
                            let ch = &app.review_groups[g_idx].channels[c_idx];
                            let check_state = app.channel_selection_state(g_idx, c_idx);
                            let (full_rect, resp) =
                                ui.allocate_exact_size(vec2(avail_w, row_h), Sense::click());
                            let rect = Rect::from_min_max(
                                pos2(full_rect.left() + 22.0, full_rect.top()),
                                full_rect.max,
                            );
                            let cb_hit = Rect::from_min_size(rect.min, vec2(34.0, row_h));

                            if ui.is_rect_visible(rect) {
                                let hovered = resp.hovered();
                                let painter = ui.painter();
                                if hovered {
                                    painter.rect_filled(rect, CornerRadius::same(RADIUS), p.bg_2);
                                }

                                let cb_rect = Rect::from_center_size(
                                    pos2(rect.left() + 16.0, rect.center().y),
                                    vec2(14.0, 14.0),
                                );
                                theme::paint_tristate_checkbox(painter, cb_rect, check_state, p, 1.0);

                                let prefix = if ch.is_dm { "@" } else { "#" };
                                let title = format!("{prefix} {}", ch.name);
                                let fg = if hovered { p.text_0 } else { p.text_1 };
                                painter.text(
                                    pos2(cb_rect.right() + 10.0, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    title,
                                    FontId::proportional(12.5),
                                    fg,
                                );

                                let chev = Rect::from_center_size(
                                    pos2(rect.right() - 16.0, rect.center().y),
                                    vec2(13.0, 13.0),
                                );
                                icons::paint(
                                    painter,
                                    chev,
                                    if ch.expanded { Icon::ChevronDown } else { Icon::ChevronRight },
                                    p.text_2,
                                );

                                let badge_txt = format_number(ch.message_indices.len());
                                let badge_galley = painter.layout_no_wrap(
                                    badge_txt,
                                    FontId::proportional(11.0),
                                    p.text_2,
                                );
                                let badge_w = badge_galley.size().x + 14.0;
                                let badge_rect = Rect::from_center_size(
                                    pos2(chev.left() - badge_w * 0.5 - 8.0, rect.center().y),
                                    vec2(badge_w, 18.0),
                                );
                                painter.rect_filled(badge_rect, CornerRadius::same(9), p.bg_3);
                                painter.galley(
                                    pos2(
                                        badge_rect.center().x - badge_galley.size().x * 0.5,
                                        badge_rect.center().y - badge_galley.size().y * 0.5,
                                    ),
                                    badge_galley,
                                    p.text_2,
                                );
                            }

                            let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
                            if resp.clicked() {
                                let clicked_cb = resp
                                    .interact_pointer_pos()
                                    .is_some_and(|pos| cb_hit.contains(pos));
                                if clicked_cb {
                                    toggle_channel_check = Some((g_idx, c_idx));
                                } else {
                                    toggle_channel_expand = Some((g_idx, c_idx));
                                }
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
                                pos2(full_rect.left() + 44.0, full_rect.top()),
                                full_rect.max,
                            );

                            if ui.is_rect_visible(rect) {
                                let painter = ui.painter();
                                if !already_deleted && resp.hovered() {
                                    painter.rect_filled(rect, CornerRadius::same(6), p.bg_3);
                                }

                                let alpha = if already_deleted { 0.42 } else { 1.0 };
                                let cb_rect = Rect::from_center_size(
                                    pos2(rect.left() + 14.0, rect.center().y),
                                    vec2(14.0, 14.0),
                                );
                                let st = if checked { TriState::Checked } else { TriState::Unchecked };
                                theme::paint_tristate_checkbox(painter, cb_rect, st, p, alpha);

                                let time_x = cb_rect.right() + 10.0;
                                let time_color = p.text_2.gamma_multiply(alpha);
                                let time_galley = painter.layout_no_wrap(
                                    msg.formatted_time.clone(),
                                    FontId::monospace(11.0),
                                    time_color,
                                );
                                let time_w = time_galley.size().x;
                                painter.galley(
                                    pos2(time_x, rect.center().y - time_galley.size().y * 0.5),
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
                                painter.with_clip_rect(clip_rect).text(
                                    pos2(content_x, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    preview,
                                    FontId::proportional(12.5),
                                    p.text_1.gamma_multiply(alpha),
                                );
                            }

                            if already_deleted {
                                resp.on_hover_text(&already_deleted_tip);
                            } else if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                                toggle_msg = Some(msg.id.clone());
                            }
                        }
                    }
                }
            });
    });

    if let Some(g_idx) = toggle_group_expand {
        app.review_groups[g_idx].expanded = !app.review_groups[g_idx].expanded;
        app.rebuild_visible_rows();
    }
    if let Some(g_idx) = toggle_group_check {
        app.toggle_group_selection(g_idx);
    }
    if let Some((g_idx, c_idx)) = toggle_channel_expand {
        let ch = &mut app.review_groups[g_idx].channels[c_idx];
        ch.expanded = !ch.expanded;
        app.rebuild_visible_rows();
    }
    if let Some((g_idx, c_idx)) = toggle_channel_check {
        app.toggle_channel_selection(g_idx, c_idx);
    }
    if let Some(id) = toggle_msg
        && !app.selected_ids.remove(&id)
    {
        app.selected_ids.insert(id);
    }

    ui.add_space(14.0);
    let selected_now = app.selected_ids.len();
    let delete_label = if selected_now > 0 {
        app.i18n.fmt(
            "review.deleteCountBtn",
            &[("count", &format_number(selected_now))],
        )
    } else {
        app.i18n.t("review.deleteBtn").to_string()
    };
    let can_delete = selected_now > 0 && !app.progress.running;

    ui.horizontal_wrapped(|ui| {
        render_active_safety_chips(app, ui, p);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if theme::danger_button(ui, p, Some(Icon::Trash), &delete_label, can_delete).clicked() {
                app.request_start_deletion();
            }
        });
    });
}
