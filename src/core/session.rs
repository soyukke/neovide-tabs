use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::layout::{PaneId, PaneLayout, SplitAxis};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct SessionState {
    #[serde(default)]
    pub active_tab: usize,
    #[serde(default)]
    pub tabs: Vec<SessionTabState>,
}

impl SessionState {
    pub fn load(path: Option<&Path>) -> Result<Option<Self>> {
        let Some(path) = path else {
            return Ok(None);
        };

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read session {}", path.display()))?;
        let state = toml::from_str(&contents)
            .with_context(|| format!("failed to parse session {}", path.display()))?;
        Ok(Some(state))
    }

    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let Some(path) = path else {
            return Ok(());
        };

        if self.tabs.is_empty() {
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create session dir {}", parent.display()))?;
        }

        let contents = toml::to_string_pretty(self).context("failed to serialize session")?;
        let tmp_path = path.with_extension("toml.tmp");
        fs::write(&tmp_path, contents)
            .with_context(|| format!("failed to write session {}", tmp_path.display()))?;
        fs::rename(&tmp_path, path)
            .with_context(|| format!("failed to replace session {}", path.display()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SessionTabState {
    pub title: String,
    #[serde(default)]
    pub active_pane: usize,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub panes: Vec<SessionPaneState>,
    pub layout: StoredPaneLayout,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SessionPaneState {
    pub id: usize,
    pub cwd: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoredPaneLayout {
    Leaf {
        pane: usize,
    },
    Split {
        axis: StoredSplitAxis,
        first: Box<StoredPaneLayout>,
        second: Box<StoredPaneLayout>,
    },
}

impl StoredPaneLayout {
    pub fn to_runtime(&self) -> PaneLayout {
        match self {
            Self::Leaf { pane } => PaneLayout::Leaf(PaneId(*pane)),
            Self::Split {
                axis,
                first,
                second,
            } => PaneLayout::Split {
                axis: axis.to_axis(),
                first: Box::new(first.to_runtime()),
                second: Box::new(second.to_runtime()),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoredSplitAxis {
    Vertical,
    Horizontal,
}

impl StoredSplitAxis {
    pub(crate) fn from_axis(axis: SplitAxis) -> Self {
        match axis {
            SplitAxis::Vertical => Self::Vertical,
            SplitAxis::Horizontal => Self::Horizontal,
        }
    }

    pub(crate) fn to_axis(self) -> SplitAxis {
        match self {
            Self::Vertical => SplitAxis::Vertical,
            Self::Horizontal => SplitAxis::Horizontal,
        }
    }
}
