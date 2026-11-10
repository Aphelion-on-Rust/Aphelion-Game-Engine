//! Draws the dock layout: tab groups, dividers you can drag, and drag-to-dock for tabs.
//! It only reads the layout. Every change is pushed as a command.

use super::commands::UiCommand;
use super::models::EditorModels;
use super::panels::{self, panel_title, PanelViews};
use super::theme_apply::{to_color32, with_alpha};
use crate::layout::{Axis, DockNode, DropZone, PanelKind};
use crate::theme::query::active_theme;
use crate::theme::ThemeColors;
use eframe::egui::{self, pos2, vec2, Align2, FontId, Id, Pos2, Rect, Sense, UiBuilder};
use flux_engine::mivc::Model;

const TAB_HEIGHT: f32 = 26.0;
const DIVIDER: f32 = 4.0;

/// Where a tab group was drawn this frame. Used to find the drop target.
#[derive(Clone)]
struct GroupRect {
    rect: Rect,
    panels: Vec<PanelKind>,
}

/// View state only: which tab is being dragged, and where the groups are.
#[derive(Default)]
pub struct DockView {
    dragging: Option<PanelKind>,
    groups: Vec<GroupRect>,
}

impl DockView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        models: &EditorModels,
        views: &mut PanelViews,
        commands: &mut Vec<UiCommand>,
    ) {
        let area = ui.available_rect_before_wrap();
        let colors = &active_theme(models.theme.data()).colors;

        self.groups.clear();
        self.draw_node(ui, models, views, colors, &models.layout.data().root, area, &mut Vec::new(), commands);
        self.handle_drag(ui, colors, commands);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_node(
        &mut self,
        ui: &mut egui::Ui,
        models: &EditorModels,
        views: &mut PanelViews,
        colors: &ThemeColors,
        node: &DockNode,
        rect: Rect,
        path: &mut Vec<bool>,
        commands: &mut Vec<UiCommand>,
    ) {
        match node {
            DockNode::Split { axis, ratio, first, second } => {
                let (first_rect, divider_rect, second_rect) = split_rects(rect, *axis, *ratio);

                path.push(false);
                self.draw_node(ui, models, views, colors, first, first_rect, path, commands);
                path.pop();

                path.push(true);
                self.draw_node(ui, models, views, colors, second, second_rect, path, commands);
                path.pop();

                draw_divider(ui, colors, *axis, *ratio, rect, divider_rect, path, commands);
            }
            DockNode::Tabs { panels, active } => {
                self.draw_tabs(ui, models, views, colors, rect, panels, *active, commands);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_tabs(
        &mut self,
        ui: &mut egui::Ui,
        models: &EditorModels,
        views: &mut PanelViews,
        colors: &ThemeColors,
        rect: Rect,
        panels_in_group: &[PanelKind],
        active: usize,
        commands: &mut Vec<UiCommand>,
    ) {
        let header = Rect::from_min_size(rect.min, vec2(rect.width(), TAB_HEIGHT));
        let content = Rect::from_min_max(pos2(rect.left(), header.bottom()), rect.max);

        ui.painter().rect_filled(header, 0.0, to_color32(colors.panel_header));
        ui.painter().rect_filled(content, 0.0, to_color32(colors.panel));
        self.groups.push(GroupRect { rect, panels: panels_in_group.to_vec() });

        let mut x = header.left() + 4.0;

        for (index, panel) in panels_in_group.iter().enumerate() {
            let is_active = index == active;
            let text_color = if is_active { colors.text } else { colors.text_dim };
            let galley = ui.fonts(|fonts| {
                fonts.layout_no_wrap(panel_title(*panel).to_string(), FontId::proportional(13.0), to_color32(text_color))
            });
            let tab_rect = Rect::from_min_size(pos2(x, header.top() + 2.0), vec2(galley.size().x + 34.0, TAB_HEIGHT - 2.0));
            let tab_id = format!("{panel:?}");

            let response = ui.interact(tab_rect, Id::new(("tab", tab_id.clone())), Sense::click_and_drag());

            if is_active {
                ui.painter().rect_filled(tab_rect, 3.0, to_color32(colors.panel));
                ui.painter().rect_filled(
                    Rect::from_min_size(tab_rect.left_top(), vec2(tab_rect.width(), 2.0)),
                    0.0,
                    to_color32(colors.accent),
                );
            }

            let text_pos = pos2(tab_rect.left() + 8.0, tab_rect.center().y - galley.size().y / 2.0);
            ui.painter().galley(text_pos, galley, to_color32(text_color));

            // The small x that closes the panel.
            let close_rect = Rect::from_center_size(pos2(tab_rect.right() - 12.0, tab_rect.center().y), vec2(14.0, 14.0));
            let close = ui.interact(close_rect, Id::new(("close", tab_id)), Sense::click());

            ui.painter().text(
                close_rect.center(),
                Align2::CENTER_CENTER,
                "x",
                FontId::proportional(12.0),
                to_color32(if close.hovered() { colors.error } else { colors.text_dim }),
            );

            if close.clicked() {
                commands.push(UiCommand::ClosePanel(*panel));
            } else if response.clicked() {
                commands.push(UiCommand::SetActiveTab(*panel));
            }

            if response.drag_started() {
                self.dragging = Some(*panel);
            }

            x += tab_rect.width() + 2.0;
        }

        if let Some(panel) = panels_in_group.get(active.min(panels_in_group.len().saturating_sub(1))) {
            ui.allocate_new_ui(UiBuilder::new().max_rect(content.shrink(6.0)), |ui| {
                ui.set_clip_rect(content);
                panels::show_panel(*panel, ui, models, views, commands);
            });
        }
    }

    /// While a tab is being dragged: highlight where it would land, and drop it on release.
    fn handle_drag(&mut self, ui: &mut egui::Ui, colors: &ThemeColors, commands: &mut Vec<UiCommand>) {
        let Some(panel) = self.dragging else {
            return;
        };

        let (pointer, released, escape) = ui.input(|state| {
            (state.pointer.interact_pos(), state.pointer.any_released(), state.key_pressed(egui::Key::Escape))
        });

        let Some(pointer) = pointer else {
            if released {
                self.dragging = None;
            }
            return;
        };

        if escape {
            self.dragging = None;
            return;
        }

        let group = self.groups.iter().find(|group| group.rect.contains(pointer)).cloned();

        if let Some(group) = group {
            let zone = drop_zone(group.rect, pointer);

            ui.painter().rect_filled(zone_rect(group.rect, zone), 0.0, with_alpha(colors.accent, 70));

            if released {
                // Drop on another panel of the group if there is one, so a lone tab can't target itself.
                let target = group.panels.iter().copied().find(|other| *other != panel).or_else(|| group.panels.first().copied());

                if let Some(target) = target {
                    commands.push(UiCommand::MovePanel { panel, target, zone });
                }
            }
        }

        ui.painter().text(
            pointer + vec2(14.0, 14.0),
            Align2::LEFT_TOP,
            panel_title(panel),
            FontId::proportional(13.0),
            to_color32(colors.text),
        );

        if released {
            self.dragging = None;
        }

        ui.ctx().request_repaint();
    }
}

/// The two halves of a split and the divider between them.
fn split_rects(rect: Rect, axis: Axis, ratio: f32) -> (Rect, Rect, Rect) {
    match axis {
        Axis::Horizontal => {
            let x = rect.left() + (rect.width() - DIVIDER) * ratio;

            (
                Rect::from_min_max(rect.min, pos2(x, rect.bottom())),
                Rect::from_min_max(pos2(x, rect.top()), pos2(x + DIVIDER, rect.bottom())),
                Rect::from_min_max(pos2(x + DIVIDER, rect.top()), rect.max),
            )
        }
        Axis::Vertical => {
            let y = rect.top() + (rect.height() - DIVIDER) * ratio;

            (
                Rect::from_min_max(rect.min, pos2(rect.right(), y)),
                Rect::from_min_max(pos2(rect.left(), y), pos2(rect.right(), y + DIVIDER)),
                Rect::from_min_max(pos2(rect.left(), y + DIVIDER), rect.max),
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_divider(
    ui: &mut egui::Ui,
    colors: &ThemeColors,
    axis: Axis,
    ratio: f32,
    parent: Rect,
    divider_rect: Rect,
    path: &[bool],
    commands: &mut Vec<UiCommand>,
) {
    let response = ui.interact(divider_rect, Id::new(("divider", path.to_vec())), Sense::drag());
    let active = response.hovered() || response.dragged();

    if active {
        ui.ctx().set_cursor_icon(match axis {
            Axis::Horizontal => egui::CursorIcon::ResizeHorizontal,
            Axis::Vertical => egui::CursorIcon::ResizeVertical,
        });
    }

    ui.painter().rect_filled(divider_rect, 0.0, to_color32(if active { colors.accent } else { colors.border }));

    if response.dragged() {
        let (total, delta) = match axis {
            Axis::Horizontal => (parent.width() - DIVIDER, response.drag_delta().x),
            Axis::Vertical => (parent.height() - DIVIDER, response.drag_delta().y),
        };

        if total > 0.0 {
            commands.push(UiCommand::SetSplitRatio { path: path.to_vec(), ratio: ratio + delta / total });
        }
    }
}

/// Edges of the group make a split, the middle makes a tab.
fn drop_zone(rect: Rect, pointer: Pos2) -> DropZone {
    let across = (pointer.x - rect.left()) / rect.width();
    let down = (pointer.y - rect.top()) / rect.height();

    if across < 0.25 {
        DropZone::Left
    } else if across > 0.75 {
        DropZone::Right
    } else if down < 0.25 {
        DropZone::Top
    } else if down > 0.75 {
        DropZone::Bottom
    } else {
        DropZone::Center
    }
}

/// The part of the group that lights up for a zone.
fn zone_rect(rect: Rect, zone: DropZone) -> Rect {
    let middle = rect.center();

    match zone {
        DropZone::Left => Rect::from_min_max(rect.min, pos2(middle.x, rect.bottom())),
        DropZone::Right => Rect::from_min_max(pos2(middle.x, rect.top()), rect.max),
        DropZone::Top => Rect::from_min_max(rect.min, pos2(rect.right(), middle.y)),
        DropZone::Bottom => Rect::from_min_max(pos2(rect.left(), middle.y), rect.max),
        DropZone::Center => rect,
    }
}
