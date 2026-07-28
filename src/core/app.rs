use serde::Serialize;

use super::layout::{PaneId, PaneLayout, PaneLayoutSnapshot, SplitAxis};

const DEFAULT_THEME_NAME: &str = "Graphite";

#[derive(Clone, Debug, PartialEq)]
pub struct TerminalCore {
    tabs: Vec<TerminalCoreTab>,
    active_tab: usize,
    next_tab_id: usize,
    next_pane_id: usize,
}

impl Default for TerminalCore {
    fn default() -> Self {
        let mut core = Self {
            tabs: Vec::new(),
            active_tab: 0,
            next_tab_id: 1,
            next_pane_id: 1,
        };
        core.new_tab();
        core
    }
}

impl TerminalCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_tab(&mut self) -> usize {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        let pane_id = self.alloc_pane_id();
        let theme = self
            .active_tab()
            .map(|tab| tab.theme.clone())
            .unwrap_or_else(|| DEFAULT_THEME_NAME.to_owned());
        self.tabs.push(TerminalCoreTab::new(tab_id, pane_id, theme));
        self.active_tab = self.tabs.len() - 1;
        self.active_tab
    }

    pub fn split_active(&mut self, axis: SplitAxis) -> Option<usize> {
        let pane_id = self.alloc_pane_id();
        self.active_tab_mut()?.split_active(pane_id, axis);
        Some(pane_id.0)
    }

    pub fn close_pane(&mut self, pane_id: usize) -> bool {
        let pane_id = PaneId(pane_id);
        let Some(tab_index) = self.tabs.iter().position(|tab| tab.contains_pane(pane_id)) else {
            return false;
        };
        if self.tabs[tab_index].pane_count() == 1 {
            self.tabs.remove(tab_index);
            self.select_neighbor_after_tab_close(tab_index);
            return true;
        }
        self.tabs[tab_index].close_pane(pane_id)
    }

    pub fn select_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        self.active_tab = index;
        true
    }

    pub fn select_pane(&mut self, pane_id: usize) -> bool {
        let pane_id = PaneId(pane_id);
        let Some(tab) = self.active_tab_mut() else {
            return false;
        };
        if !tab.contains_pane(pane_id) {
            return false;
        }
        tab.active_pane = pane_id;
        true
    }

    pub fn rename_tab(&mut self, index: usize, title: impl Into<String>) -> bool {
        let Some(tab) = self.tabs.get_mut(index) else {
            return false;
        };
        let title = title.into();
        if title.trim().is_empty() {
            return false;
        }
        tab.title = title;
        true
    }

    pub fn set_tab_theme(&mut self, index: usize, theme: impl Into<String>) -> bool {
        let Some(tab) = self.tabs.get_mut(index) else {
            return false;
        };
        tab.theme = theme.into();
        true
    }

    pub fn snapshot(&self) -> TerminalCoreSnapshot {
        TerminalCoreSnapshot {
            active_tab: self.active_tab,
            tabs: self
                .tabs
                .iter()
                .enumerate()
                .map(|(index, tab)| tab.snapshot(index))
                .collect(),
        }
    }

    fn active_tab(&self) -> Option<&TerminalCoreTab> {
        self.tabs.get(self.active_tab)
    }

    fn active_tab_mut(&mut self) -> Option<&mut TerminalCoreTab> {
        self.tabs.get_mut(self.active_tab)
    }

    fn alloc_pane_id(&mut self) -> PaneId {
        let id = PaneId(self.next_pane_id);
        self.next_pane_id += 1;
        id
    }

    fn select_neighbor_after_tab_close(&mut self, closed_index: usize) {
        if self.tabs.is_empty() {
            self.active_tab = 0;
        } else if closed_index < self.active_tab {
            self.active_tab -= 1;
        } else if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TerminalCoreTab {
    title: String,
    active_pane: PaneId,
    theme: String,
    panes: Vec<PaneId>,
    layout: PaneLayout,
}

impl TerminalCoreTab {
    fn new(tab_id: usize, pane_id: PaneId, theme: String) -> Self {
        Self {
            title: format!("session {tab_id}"),
            active_pane: pane_id,
            theme,
            panes: vec![pane_id],
            layout: PaneLayout::Leaf(pane_id),
        }
    }

    fn split_active(&mut self, pane_id: PaneId, axis: SplitAxis) {
        if self.layout.split_leaf(self.active_pane, pane_id, axis) {
            self.panes.push(pane_id);
            self.active_pane = pane_id;
        }
    }

    fn close_pane(&mut self, pane_id: PaneId) -> bool {
        if self.pane_count() <= 1 || !self.contains_pane(pane_id) {
            return false;
        }
        let Some(layout) = self.layout.clone().without_leaf(pane_id) else {
            return false;
        };
        self.layout = layout;
        self.panes.retain(|id| *id != pane_id);
        if self.active_pane == pane_id {
            self.active_pane = self.layout.first_leaf().unwrap_or(self.panes[0]);
        }
        true
    }

    fn contains_pane(&self, pane_id: PaneId) -> bool {
        self.panes.contains(&pane_id)
    }

    fn pane_count(&self) -> usize {
        self.panes.len()
    }

    fn snapshot(&self, index: usize) -> TerminalCoreTabSnapshot {
        TerminalCoreTabSnapshot {
            index,
            title: self.title.clone(),
            active_pane: self.active_pane.0,
            theme: self.theme.clone(),
            panes: self.panes.iter().map(|pane| pane.0).collect(),
            layout: self.layout.snapshot(),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TerminalCoreSnapshot {
    pub active_tab: usize,
    pub tabs: Vec<TerminalCoreTabSnapshot>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TerminalCoreTabSnapshot {
    pub index: usize,
    pub title: String,
    pub active_pane: usize,
    pub theme: String,
    pub panes: Vec<usize>,
    pub layout: PaneLayoutSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_core_starts_with_one_session_tab() {
        let snapshot = TerminalCore::new().snapshot();

        assert_eq!(snapshot.active_tab, 0);
        assert_eq!(snapshot.tabs[0].title, "session 1");
        assert_eq!(snapshot.tabs[0].active_pane, 1);
    }

    #[test]
    fn splitting_active_pane_selects_the_new_pane() {
        let mut core = TerminalCore::new();

        assert_eq!(core.split_active(SplitAxis::Vertical), Some(2));
        let snapshot = core.snapshot();
        assert_eq!(snapshot.tabs[0].active_pane, 2);
        assert_eq!(
            serde_json::to_value(&snapshot.tabs[0].layout).unwrap(),
            serde_json::json!({
                "kind": "split",
                "axis": "vertical",
                "first": {"kind": "leaf", "pane_id": 1},
                "second": {"kind": "leaf", "pane_id": 2}
            })
        );
    }

    #[test]
    fn selecting_a_visible_split_pane_updates_focus() {
        let mut core = TerminalCore::new();
        assert_eq!(core.split_active(SplitAxis::Horizontal), Some(2));

        assert!(core.select_pane(1));
        assert_eq!(core.snapshot().tabs[0].active_pane, 1);
        assert!(!core.select_pane(99));
        assert_eq!(core.snapshot().tabs[0].active_pane, 1);
    }

    #[test]
    fn closing_split_pane_selects_the_remaining_pane() {
        let mut core = TerminalCore::new();
        assert_eq!(core.split_active(SplitAxis::Vertical), Some(2));

        assert!(core.close_pane(2));
        assert_eq!(core.snapshot().tabs[0].active_pane, 1);
    }

    #[test]
    fn closing_last_pane_removes_tab_and_selects_neighbor() {
        let mut core = TerminalCore::new();
        assert_eq!(core.new_tab(), 1);

        assert!(core.close_pane(2));
        let snapshot = core.snapshot();

        assert_eq!(snapshot.tabs.len(), 1);
        assert_eq!(snapshot.active_tab, 0);
        assert_eq!(snapshot.tabs[0].active_pane, 1);
    }
}
