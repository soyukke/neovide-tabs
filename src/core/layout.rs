use super::session::{StoredPaneLayout, StoredSplitAxis};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PaneId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaneLayout {
    Leaf(PaneId),
    Split {
        axis: SplitAxis,
        first: Box<PaneLayout>,
        second: Box<PaneLayout>,
    },
}

impl PaneLayout {
    pub fn split_leaf(&mut self, target: PaneId, new_pane: PaneId, axis: SplitAxis) -> bool {
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

    pub fn without_leaf(self, target: PaneId) -> Option<Self> {
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

    pub fn first_leaf(&self) -> Option<PaneId> {
        match self {
            Self::Leaf(id) => Some(*id),
            Self::Split { first, .. } => first.first_leaf(),
        }
    }

    pub fn collect_with<R, F>(&self, rect: R, split: &F, out: &mut Vec<(PaneId, R)>)
    where
        R: Copy,
        F: Fn(R, SplitAxis) -> (R, R),
    {
        match self {
            Self::Leaf(id) => out.push((*id, rect)),
            Self::Split {
                axis,
                first,
                second,
            } => {
                let (first_rect, second_rect) = split(rect, *axis);
                first.collect_with(first_rect, split, out);
                second.collect_with(second_rect, split, out);
            }
        }
    }

    pub fn contains_only(&self, pane_ids: &[PaneId]) -> bool {
        match self {
            Self::Leaf(id) => pane_ids.contains(id),
            Self::Split { first, second, .. } => {
                first.contains_only(pane_ids) && second.contains_only(pane_ids)
            }
        }
    }

    pub fn to_stored(&self) -> StoredPaneLayout {
        match self {
            Self::Leaf(id) => StoredPaneLayout::Leaf { pane: id.0 },
            Self::Split {
                axis,
                first,
                second,
            } => StoredPaneLayout::Split {
                axis: StoredSplitAxis::from_axis(*axis),
                first: Box::new(first.to_stored()),
                second: Box::new(second.to_stored()),
            },
        }
    }
}
