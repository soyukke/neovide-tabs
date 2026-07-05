use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{
    layout::{PaneId, PaneLayout, SplitAxis},
    session::{SessionPaneState, SessionState, SessionTabState},
};

pub const DEFAULT_THEME_NAME: &str = "Graphite";

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

    pub fn from_session(session: &SessionState) -> Self {
        let mut core = Self::empty();
        for tab in &session.tabs {
            if let Some(tab) = TerminalCoreTab::from_session(tab) {
                core.tabs.push(tab);
            }
        }

        if core.tabs.is_empty() {
            core.new_tab();
        } else {
            core.active_tab = session.active_tab.min(core.tabs.len() - 1);
            core.recalculate_next_ids();
        }
        core
    }

    pub fn new_tab(&mut self) -> usize {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        let pane_id = self.alloc_pane_id();
        let theme = self
            .active_tab()
            .map(|tab| tab.theme.clone())
            .unwrap_or_else(|| DEFAULT_THEME_NAME.to_owned());
        let tab = TerminalCoreTab::new(tab_id, pane_id, theme);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.active_tab
    }

    pub fn split_active(&mut self, axis: SplitAxis) -> Option<usize> {
        let pane_id = self.alloc_pane_id();
        let tab = self.active_tab_mut()?;
        tab.split_active(pane_id, axis);
        Some(pane_id.0)
    }

    pub fn select_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        self.active_tab = index;
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

    pub fn to_session(&self) -> SessionState {
        SessionState {
            active_tab: self.active_tab.min(self.tabs.len().saturating_sub(1)),
            tabs: self.tabs.iter().map(TerminalCoreTab::to_session).collect(),
        }
    }

    fn empty() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            next_tab_id: 1,
            next_pane_id: 1,
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

    fn recalculate_next_ids(&mut self) {
        self.next_tab_id = self
            .tabs
            .iter()
            .filter_map(|tab| tab.title.strip_prefix("session ")?.parse::<usize>().ok())
            .max()
            .unwrap_or(self.tabs.len())
            + 1;
        self.next_pane_id = self
            .tabs
            .iter()
            .flat_map(|tab| tab.panes.iter().map(|pane| pane.id.0))
            .max()
            .unwrap_or(0)
            + 1;
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TerminalCoreTab {
    title: String,
    active_pane: PaneId,
    theme: String,
    panes: Vec<TerminalCorePane>,
    layout: PaneLayout,
}

impl TerminalCoreTab {
    fn new(tab_id: usize, pane_id: PaneId, theme: String) -> Self {
        Self {
            title: format!("session {tab_id}"),
            active_pane: pane_id,
            theme,
            panes: vec![TerminalCorePane {
                id: pane_id,
                cwd: None,
            }],
            layout: PaneLayout::Leaf(pane_id),
        }
    }

    fn from_session(state: &SessionTabState) -> Option<Self> {
        let panes = session_panes(state);
        let first_pane = panes.first()?.id;
        let pane_ids = panes.iter().map(|pane| pane.id).collect::<Vec<_>>();
        let layout = valid_layout_or_leaf(state.layout.to_runtime(), &pane_ids, first_pane);
        let active_pane = valid_active_pane(PaneId(state.active_pane), &pane_ids, &layout);

        Some(Self {
            title: state.title.clone(),
            active_pane,
            theme: normalized_theme(&state.theme),
            panes,
            layout,
        })
    }

    fn split_active(&mut self, pane_id: PaneId, axis: SplitAxis) {
        if self.layout.split_leaf(self.active_pane, pane_id, axis) {
            self.panes.push(TerminalCorePane {
                id: pane_id,
                cwd: None,
            });
            self.active_pane = pane_id;
        }
    }

    fn snapshot(&self, index: usize) -> TerminalCoreTabSnapshot {
        TerminalCoreTabSnapshot {
            index,
            title: self.title.clone(),
            active_pane: self.active_pane.0,
            theme: self.theme.clone(),
            panes: self.panes.iter().map(TerminalCorePane::snapshot).collect(),
            layout: self.layout.to_stored(),
        }
    }

    fn to_session(&self) -> SessionTabState {
        SessionTabState {
            title: self.title.clone(),
            active_pane: self.active_pane.0,
            theme: self.theme.clone(),
            panes: self
                .panes
                .iter()
                .map(TerminalCorePane::to_session)
                .collect(),
            layout: self.layout.to_stored(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TerminalCorePane {
    id: PaneId,
    cwd: Option<PathBuf>,
}

impl TerminalCorePane {
    fn snapshot(&self) -> TerminalCorePaneSnapshot {
        TerminalCorePaneSnapshot {
            id: self.id.0,
            cwd: self.cwd.clone(),
        }
    }

    fn to_session(&self) -> SessionPaneState {
        SessionPaneState {
            id: self.id.0,
            cwd: self.cwd.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TerminalCoreSnapshot {
    pub active_tab: usize,
    pub tabs: Vec<TerminalCoreTabSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TerminalCoreTabSnapshot {
    pub index: usize,
    pub title: String,
    pub active_pane: usize,
    pub theme: String,
    pub panes: Vec<TerminalCorePaneSnapshot>,
    pub layout: super::session::StoredPaneLayout,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TerminalCorePaneSnapshot {
    pub id: usize,
    pub cwd: Option<PathBuf>,
}

fn session_panes(state: &SessionTabState) -> Vec<TerminalCorePane> {
    let mut panes = Vec::new();
    for pane in &state.panes {
        let id = PaneId(pane.id);
        if id.0 == 0
            || panes
                .iter()
                .any(|existing: &TerminalCorePane| existing.id == id)
        {
            continue;
        }
        panes.push(TerminalCorePane {
            id,
            cwd: pane.cwd.clone(),
        });
    }
    panes
}

fn valid_layout_or_leaf(layout: PaneLayout, pane_ids: &[PaneId], fallback: PaneId) -> PaneLayout {
    if layout.contains_only(pane_ids) {
        layout
    } else {
        PaneLayout::Leaf(fallback)
    }
}

fn valid_active_pane(active_pane: PaneId, pane_ids: &[PaneId], layout: &PaneLayout) -> PaneId {
    if pane_ids.contains(&active_pane) {
        active_pane
    } else {
        layout.first_leaf().unwrap_or(pane_ids[0])
    }
}

fn normalized_theme(theme: &str) -> String {
    if theme.trim().is_empty() {
        DEFAULT_THEME_NAME.to_owned()
    } else {
        theme.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{StoredPaneLayout, StoredSplitAxis};

    #[test]
    fn default_core_starts_with_one_session_tab() {
        let snapshot = TerminalCore::new().snapshot();

        assert_eq!(snapshot.active_tab, 0);
        assert_eq!(snapshot.tabs[0].title, "session 1");
        assert_eq!(snapshot.tabs[0].panes[0].id, 1);
    }

    #[test]
    fn splitting_active_pane_updates_layout_and_active_pane() {
        let mut core = TerminalCore::new();

        assert_eq!(core.split_active(SplitAxis::Vertical), Some(2));
        let snapshot = core.snapshot();

        assert_eq!(snapshot.tabs[0].active_pane, 2);
        assert_eq!(snapshot.tabs[0].panes.len(), 2);
        assert!(matches!(
            snapshot.tabs[0].layout,
            StoredPaneLayout::Split {
                axis: StoredSplitAxis::Vertical,
                ..
            }
        ));
    }

    #[test]
    fn session_restore_preserves_next_ids() {
        let session = TerminalCore::new().to_session();
        let mut core = TerminalCore::from_session(&session);

        let tab_index = core.new_tab();
        assert_eq!(tab_index, 1);
        assert_eq!(core.snapshot().tabs[1].title, "session 2");
    }
}
