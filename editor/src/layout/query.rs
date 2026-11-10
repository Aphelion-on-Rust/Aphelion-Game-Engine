use super::interface::{LayoutData, PanelKind};
use super::tree;

pub fn all_panels(data: &LayoutData) -> Vec<PanelKind> {
    tree::all_panels(&data.root)
}

pub fn is_open(data: &LayoutData, panel: PanelKind) -> bool {
    all_panels(data).contains(&panel)
}
