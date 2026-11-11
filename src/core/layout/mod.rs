use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ContainerKind {
    Selection,    // 1: Seleção (Letras, Mídias, Pastas)
    Lyrics,       // 2: Visualização da Letra / Estrófes
    Stage,        // 3: Retorno Ao Vivo
    Preview,      // 4: Visualização (Prévia do slide)
    Themes,       // 5: Temas
    Playlist,     // 6: Lista de reprodução (Status já tocada no culto)
    Player,       // 7: Player (Vídeos e Áudio)
    ChatChannels, // 8: Canais e Conversas
    ChatMessages, // 9: Mensagens do Chat
}

impl ContainerKind {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Selection => "workspace-selection",
            Self::Lyrics => "workspace-lyrics-preview",
            Self::Stage => "workspace-live-output",
            Self::Preview => "workspace-preview-label",
            Self::Themes => "workspace-themes",
            Self::Playlist => "workspace-playlist",
            Self::Player => "control-player",
            Self::ChatChannels => "chat-channels",
            Self::ChatMessages => "chat-messages",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropZone {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LayoutNode {
    Leaf(ContainerKind),
    Split {
        direction: SplitDirection,
        fraction: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

impl LayoutNode {
    pub const MIN_LEAF_WIDTH: f32 = 180.0;
    pub const MIN_LEAF_HEIGHT: f32 = 90.0;
    pub const SPLITTER_SIZE: f32 = 6.0;

    /// Recursively clamps all split fractions to safe bounds so corrupt or extreme states auto-heal.
    pub fn sanitize_fractions(&mut self) {
        if let LayoutNode::Split {
            fraction,
            first,
            second,
            ..
        } = self
        {
            *fraction = (*fraction).clamp(0.15, 0.85);
            first.sanitize_fractions();
            second.sanitize_fractions();
        }
    }

    /// Calculates the minimum width required by this subtree so no container is squashed.
    pub fn min_width(&self) -> f32 {
        match self {
            LayoutNode::Leaf(_) => Self::MIN_LEAF_WIDTH,
            LayoutNode::Split {
                direction,
                first,
                second,
                ..
            } => match direction {
                SplitDirection::Horizontal => {
                    first.min_width() + second.min_width() + Self::SPLITTER_SIZE
                }
                SplitDirection::Vertical => first.min_width().max(second.min_width()),
            },
        }
    }

    /// Calculates the minimum height required by this subtree so no container is squashed.
    pub fn min_height(&self) -> f32 {
        match self {
            LayoutNode::Leaf(_) => Self::MIN_LEAF_HEIGHT,
            LayoutNode::Split {
                direction,
                first,
                second,
                ..
            } => match direction {
                SplitDirection::Vertical => {
                    first.min_height() + second.min_height() + Self::SPLITTER_SIZE
                }
                SplitDirection::Horizontal => first.min_height().max(second.min_height()),
            },
        }
    }

    pub fn contains(&self, target: ContainerKind) -> bool {
        match self {
            LayoutNode::Leaf(k) => *k == target,
            LayoutNode::Split { first, second, .. } => {
                first.contains(target) || second.contains(target)
            }
        }
    }

    pub fn collect_leaves(&self, out: &mut Vec<ContainerKind>) {
        match self {
            LayoutNode::Leaf(k) => out.push(*k),
            LayoutNode::Split { first, second, .. } => {
                first.collect_leaves(out);
                second.collect_leaves(out);
            }
        }
    }

    pub fn swap_leaves(&mut self, a: ContainerKind, b: ContainerKind) {
        match self {
            LayoutNode::Leaf(k) => {
                if *k == a {
                    *k = b;
                } else if *k == b {
                    *k = a;
                }
            }
            LayoutNode::Split { first, second, .. } => {
                first.swap_leaves(a, b);
                second.swap_leaves(a, b);
            }
        }
    }

    pub fn remove_leaf(&mut self, target: ContainerKind) -> bool {
        match self {
            LayoutNode::Leaf(k) => *k == target,
            LayoutNode::Split { first, second, .. } => {
                if let LayoutNode::Leaf(k) = **first
                    && k == target {
                        let s = (**second).clone();
                        *self = s;
                        return true;
                    }
                if let LayoutNode::Leaf(k) = **second
                    && k == target {
                        let f = (**first).clone();
                        *self = f;
                        return true;
                    }
                if first.remove_leaf(target) {
                    return true;
                }
                second.remove_leaf(target)
            }
        }
    }

    pub fn dock_at(
        &mut self,
        dragged: ContainerKind,
        target: ContainerKind,
        zone: DropZone,
    ) -> bool {
        match self {
            LayoutNode::Leaf(k) => {
                if *k == target {
                    match zone {
                        DropZone::Center => {
                            *self = LayoutNode::Leaf(dragged);
                        }
                        DropZone::Left => {
                            *self = LayoutNode::Split {
                                direction: SplitDirection::Horizontal,
                                fraction: 0.5,
                                first: Box::new(LayoutNode::Leaf(dragged)),
                                second: Box::new(LayoutNode::Leaf(target)),
                            };
                        }
                        DropZone::Right => {
                            *self = LayoutNode::Split {
                                direction: SplitDirection::Horizontal,
                                fraction: 0.5,
                                first: Box::new(LayoutNode::Leaf(target)),
                                second: Box::new(LayoutNode::Leaf(dragged)),
                            };
                        }
                        DropZone::Top => {
                            *self = LayoutNode::Split {
                                direction: SplitDirection::Vertical,
                                fraction: 0.5,
                                first: Box::new(LayoutNode::Leaf(dragged)),
                                second: Box::new(LayoutNode::Leaf(target)),
                            };
                        }
                        DropZone::Bottom => {
                            *self = LayoutNode::Split {
                                direction: SplitDirection::Vertical,
                                fraction: 0.5,
                                first: Box::new(LayoutNode::Leaf(target)),
                                second: Box::new(LayoutNode::Leaf(dragged)),
                            };
                        }
                    }
                    return true;
                }
                false
            }
            LayoutNode::Split { first, second, .. } => {
                if first.dock_at(dragged, target, zone) {
                    return true;
                }
                second.dock_at(dragged, target, zone)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayoutTree {
    pub root: LayoutNode,
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::default_4_columns()
    }
}

impl LayoutTree {
    pub fn bible_workspace() -> Self {
        Self {
            root: LayoutNode::Split {
                direction: SplitDirection::Horizontal,
                fraction: 0.20,
                first: Box::new(LayoutNode::Leaf(ContainerKind::Selection)),
                second: Box::new(LayoutNode::Split {
                    direction: SplitDirection::Horizontal,
                    fraction: 0.38,
                    first: Box::new(LayoutNode::Split {
                        direction: SplitDirection::Vertical,
                        fraction: 0.30,
                        first: Box::new(LayoutNode::Leaf(ContainerKind::Player)),
                        second: Box::new(LayoutNode::Leaf(ContainerKind::Lyrics)),
                    }),
                    second: Box::new(LayoutNode::Split {
                        direction: SplitDirection::Horizontal,
                        fraction: 0.63,
                        first: Box::new(LayoutNode::Split {
                            direction: SplitDirection::Vertical,
                            fraction: 0.68,
                            first: Box::new(LayoutNode::Split {
                                direction: SplitDirection::Vertical,
                                fraction: 0.50,
                                first: Box::new(LayoutNode::Leaf(ContainerKind::Stage)),
                                second: Box::new(LayoutNode::Leaf(ContainerKind::Preview)),
                            }),
                            second: Box::new(LayoutNode::Leaf(ContainerKind::Themes)),
                        }),
                        second: Box::new(LayoutNode::Leaf(ContainerKind::Playlist)),
                    }),
                }),
            },
        }
    }

    pub fn default_4_columns() -> Self {
        Self {
            root: LayoutNode::Split {
                direction: SplitDirection::Horizontal,
                fraction: 0.22,
                first: Box::new(LayoutNode::Leaf(ContainerKind::Selection)),
                second: Box::new(LayoutNode::Split {
                    direction: SplitDirection::Horizontal,
                    fraction: 0.38,
                    first: Box::new(LayoutNode::Split {
                        direction: SplitDirection::Vertical,
                        fraction: 0.30,
                        first: Box::new(LayoutNode::Leaf(ContainerKind::Player)),
                        second: Box::new(LayoutNode::Leaf(ContainerKind::Lyrics)),
                    }),
                    second: Box::new(LayoutNode::Split {
                        direction: SplitDirection::Horizontal,
                        fraction: 0.54,
                        first: Box::new(LayoutNode::Split {
                            direction: SplitDirection::Vertical,
                            fraction: 0.68,
                            first: Box::new(LayoutNode::Split {
                                direction: SplitDirection::Vertical,
                                fraction: 0.50,
                                first: Box::new(LayoutNode::Leaf(ContainerKind::Stage)),
                                second: Box::new(LayoutNode::Leaf(ContainerKind::Preview)),
                            }),
                            second: Box::new(LayoutNode::Leaf(ContainerKind::Themes)),
                        }),
                        second: Box::new(LayoutNode::Leaf(ContainerKind::Playlist)),
                    }),
                }),
            },
        }
    }

    pub fn default_chat() -> Self {
        Self {
            root: LayoutNode::Split {
                direction: SplitDirection::Horizontal,
                fraction: 0.24,
                first: Box::new(LayoutNode::Leaf(ContainerKind::ChatChannels)),
                second: Box::new(LayoutNode::Leaf(ContainerKind::ChatMessages)),
            },
        }
    }

    pub fn dock(&mut self, dragged: ContainerKind, target: ContainerKind, zone: DropZone) {
        if dragged == target {
            return;
        }

        if zone == DropZone::Center {
            self.root.swap_leaves(dragged, target);
            return;
        }

        if let LayoutNode::Leaf(k) = self.root
            && k == dragged {
                return;
            }

        self.root.remove_leaf(dragged);
        self.root.dock_at(dragged, target, zone);
    }
}

pub fn calculate_drop_zone(
    pos: egui::Pos2,
    rect: egui::Rect,
) -> Option<(DropZone, egui::Rect, &'static str)> {
    if !rect.contains(pos) {
        return None;
    }

    let w = rect.width();
    let h = rect.height();

    let d_left = (pos.x - rect.left()) / w;
    let d_right = (rect.right() - pos.x) / w;
    let d_top = (pos.y - rect.top()) / h;
    let d_bottom = (rect.bottom() - pos.y) / h;

    let min_dist = d_left.min(d_right).min(d_top).min(d_bottom);

    if min_dist < 0.28 {
        if min_dist == d_left {
            let indicator = egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.left() + w * 0.45, rect.bottom()),
            );
            Some((DropZone::Left, indicator, "drag-insert-left"))
        } else if min_dist == d_right {
            let indicator = egui::Rect::from_min_max(
                egui::pos2(rect.right() - w * 0.45, rect.top()),
                rect.right_bottom(),
            );
            Some((DropZone::Right, indicator, "drag-insert-right"))
        } else if min_dist == d_top {
            let indicator = egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.right(), rect.top() + h * 0.45),
            );
            Some((DropZone::Top, indicator, "drag-insert-above"))
        } else {
            let indicator = egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.bottom() - h * 0.45),
                rect.right_bottom(),
            );
            Some((DropZone::Bottom, indicator, "drag-insert-below"))
        }
    } else {
        Some((DropZone::Center, rect, "drag-swap-position"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_docking_tree() {
        let mut tree = LayoutTree::default();

        assert!(tree.root.contains(ContainerKind::Selection));
        assert!(tree.root.contains(ContainerKind::Lyrics));
        assert!(tree.root.contains(ContainerKind::Stage));
        assert!(tree.root.contains(ContainerKind::Preview));
        assert!(tree.root.contains(ContainerKind::Themes));
        assert!(tree.root.contains(ContainerKind::Playlist));
        assert!(tree.root.contains(ContainerKind::Player));

        // 1. Dock Stage above Playlist (Vertical split)
        tree.dock(ContainerKind::Stage, ContainerKind::Playlist, DropZone::Top);
        assert!(tree.root.contains(ContainerKind::Selection));
        assert!(tree.root.contains(ContainerKind::Lyrics));
        assert!(tree.root.contains(ContainerKind::Stage));
        assert!(tree.root.contains(ContainerKind::Preview));
        assert!(tree.root.contains(ContainerKind::Themes));
        assert!(tree.root.contains(ContainerKind::Playlist));
        assert!(tree.root.contains(ContainerKind::Player));

        // 2. Dock Themes below Selection (Vertical split)
        tree.dock(
            ContainerKind::Themes,
            ContainerKind::Selection,
            DropZone::Bottom,
        );
        assert!(tree.root.contains(ContainerKind::Selection));
        assert!(tree.root.contains(ContainerKind::Lyrics));
        assert!(tree.root.contains(ContainerKind::Stage));
        assert!(tree.root.contains(ContainerKind::Preview));
        assert!(tree.root.contains(ContainerKind::Themes));
        assert!(tree.root.contains(ContainerKind::Playlist));
        assert!(tree.root.contains(ContainerKind::Player));

        // 3. Swap Selection and Stage (Center drop zone)
        tree.dock(
            ContainerKind::Selection,
            ContainerKind::Stage,
            DropZone::Center,
        );
        assert!(tree.root.contains(ContainerKind::Selection));
        assert!(tree.root.contains(ContainerKind::Lyrics));
        assert!(tree.root.contains(ContainerKind::Stage));
        assert!(tree.root.contains(ContainerKind::Preview));
        assert!(tree.root.contains(ContainerKind::Themes));
        assert!(tree.root.contains(ContainerKind::Playlist));
        assert!(tree.root.contains(ContainerKind::Player));

        // 4. Test calculate_drop_zone edge detection
        let rect = egui::Rect::from_min_max(egui::pos2(100.0, 100.0), egui::pos2(500.0, 500.0));
        let (zone, _, _) = calculate_drop_zone(egui::pos2(110.0, 300.0), rect).unwrap();
        assert_eq!(zone, DropZone::Left);
        let (zone, _, _) = calculate_drop_zone(egui::pos2(490.0, 300.0), rect).unwrap();
        assert_eq!(zone, DropZone::Right);
        let (zone, _, _) = calculate_drop_zone(egui::pos2(300.0, 110.0), rect).unwrap();
        assert_eq!(zone, DropZone::Top);
        let (zone, _, _) = calculate_drop_zone(egui::pos2(300.0, 490.0), rect).unwrap();
        assert_eq!(zone, DropZone::Bottom);
        let (zone, _, _) = calculate_drop_zone(egui::pos2(300.0, 300.0), rect).unwrap();
        assert_eq!(zone, DropZone::Center);
    }

    #[test]
    fn test_layout_min_dimensions() {
        let leaf = LayoutNode::Leaf(ContainerKind::Lyrics);
        assert_eq!(leaf.min_width(), LayoutNode::MIN_LEAF_WIDTH);
        assert_eq!(leaf.min_height(), LayoutNode::MIN_LEAF_HEIGHT);

        let default_tree = LayoutTree::default();
        // 4 major columns horizontally must require at least 4 * 180 + 3 * 6 = 738 px
        assert!(default_tree.root.min_width() >= 738.0);
        // Vertically stacked items must require adequate height
        assert!(default_tree.root.min_height() >= 186.0);

        let mut extreme_tree = LayoutTree {
            root: LayoutNode::Split {
                direction: SplitDirection::Horizontal,
                fraction: 0.02,
                first: Box::new(LayoutNode::Leaf(ContainerKind::Selection)),
                second: Box::new(LayoutNode::Leaf(ContainerKind::Lyrics)),
            },
        };
        extreme_tree.root.sanitize_fractions();
        if let LayoutNode::Split { fraction, .. } = extreme_tree.root {
            assert!(fraction >= 0.15 && fraction <= 0.85);
        }
    }
}
