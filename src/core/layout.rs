use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PaneId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PaneLayoutSnapshot {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<SplitAxis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<Box<PaneLayoutSnapshot>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second: Option<Box<PaneLayoutSnapshot>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PaneLayout {
    Leaf(PaneId),
    Split {
        axis: SplitAxis,
        first: Box<PaneLayout>,
        second: Box<PaneLayout>,
    },
}

impl PaneLayout {
    pub(crate) fn snapshot(&self) -> PaneLayoutSnapshot {
        match self {
            Self::Leaf(pane_id) => PaneLayoutSnapshot {
                kind: "leaf",
                pane_id: Some(pane_id.0),
                axis: None,
                first: None,
                second: None,
            },
            Self::Split {
                axis,
                first,
                second,
            } => PaneLayoutSnapshot {
                kind: "split",
                pane_id: None,
                axis: Some(*axis),
                first: Some(Box::new(first.snapshot())),
                second: Some(Box::new(second.snapshot())),
            },
        }
    }

    pub(crate) fn split_leaf(&mut self, target: PaneId, new_pane: PaneId, axis: SplitAxis) -> bool {
        match self {
            Self::Leaf(id) if *id == target => {
                *self = Self::Split {
                    axis,
                    first: Box::new(Self::Leaf(*id)),
                    second: Box::new(Self::Leaf(new_pane)),
                };
                true
            }
            Self::Leaf(_) => false,
            Self::Split { first, second, .. } => {
                first.split_leaf(target, new_pane, axis)
                    || second.split_leaf(target, new_pane, axis)
            }
        }
    }

    pub(crate) fn without_leaf(self, target: PaneId) -> Option<Self> {
        match self {
            Self::Leaf(id) if id == target => None,
            Self::Leaf(id) => Some(Self::Leaf(id)),
            Self::Split {
                axis,
                first,
                second,
            } => match (first.without_leaf(target), second.without_leaf(target)) {
                (Some(first), Some(second)) => Some(Self::Split {
                    axis,
                    first: Box::new(first),
                    second: Box::new(second),
                }),
                (Some(layout), None) | (None, Some(layout)) => Some(layout),
                (None, None) => None,
            },
        }
    }

    pub(crate) fn first_leaf(&self) -> Option<PaneId> {
        match self {
            Self::Leaf(id) => Some(*id),
            Self::Split { first, .. } => first.first_leaf(),
        }
    }
}
