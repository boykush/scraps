use std::collections::HashSet;
use std::f32::consts::PI;

use scraps_libs::model::{key::ScrapKey, scrap::Scrap};

/// Neighbours drawn around one scrap. A wiki hub carries hundreds of
/// backlinks, and past this many the labels stop being readable however
/// they are placed, so the rest is reported as a count instead.
pub const MAX_NODES: usize = 18;

pub const NODE_R: f32 = 5.0;
const CENTER_R: f32 = 8.5;
const BASE_RADIUS: f32 = 150.0;
/// Arc length one neighbour needs before its label starts touching the next.
const NODE_GAP: f32 = 26.0;
const SPREAD_SIDE: f32 = PI * 0.66;
const SPREAD_BOTH: f32 = PI * 0.30;
const PAD_Y: f32 = 60.0;
const LABEL_OFFSET: f32 = 9.0;
const ARROW_CLEARANCE: f32 = 5.0;
/// SVG text neither wraps nor ellipsises, so a long title is cut before it
/// reaches the markup. The full title stays on the node as a tooltip.
const LABEL_CHARS: usize = 16;
/// `--font-size-xs`, the size `main.css` gives these labels. There is no text
/// engine at build time, so the box a label needs has to be estimated from it.
const LABEL_FONT_PX: f32 = 12.0;
/// A drawn label's box is taller than its font size; two rows closer than this
/// touch. Measured against the shipped font stack, not derived.
const LABEL_GAP: f32 = LABEL_FONT_PX * 1.75;
const MIN_PAD_X: f32 = 90.0;

/// Which way the link between the centre scrap and a neighbour runs.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LinkDir {
    /// The neighbour links here: a backlink.
    In,
    /// The centre scrap links out to the neighbour.
    Out,
    /// Both directions exist.
    Both,
}

#[derive(PartialEq, Clone, Debug)]
pub struct GraphNode {
    pub key: ScrapKey,
    /// The title as drawn: cut to [`LABEL_CHARS`] so it fits on one node.
    pub label: String,
    pub dir: LinkDir,
    pub x: f32,
    pub y: f32,
}

fn label_of(title: &str) -> String {
    let chars: Vec<char> = title.chars().collect();
    if chars.len() <= LABEL_CHARS {
        title.to_string()
    } else {
        format!("{}…", chars[..LABEL_CHARS].iter().collect::<String>())
    }
}

/// Wide enough for the label not to be clipped by the viewBox. The factors are
/// upper bounds measured against the shipped font stack, not exact advances —
/// they only ever decide how much empty margin the drawing carries.
fn label_width(label: &str) -> f32 {
    label
        .chars()
        .map(|c| if c.is_ascii() { 0.80 } else { 1.05 })
        .sum::<f32>()
        * LABEL_FONT_PX
}

/// A scrap and its direct neighbours, already laid out. Direction is carried
/// by position — backlinks left, links right, mutual on the vertical — so the
/// drawing stays readable without a second colour to decode.
#[derive(PartialEq, Clone, Debug)]
pub struct ScrapGraph {
    pub nodes: Vec<GraphNode>,
    /// The centre scrap's own title, cut the same way a neighbour's is.
    pub center_label: String,
    pub dropped: usize,
    pub width: f32,
    pub height: f32,
    pub center_x: f32,
    pub center_y: f32,
}

impl ScrapGraph {
    pub fn center_r(&self) -> f32 {
        CENTER_R
    }

    pub fn center_label_y(&self) -> f32 {
        self.center_y + CENTER_R + 15.0
    }
}

fn sort_key(key: &ScrapKey) -> (String, String) {
    (
        key.ctx()
            .as_ref()
            .map(|ctx| ctx.to_string())
            .unwrap_or_default(),
        key.title().to_string(),
    )
}

impl ScrapGraph {
    /// `None` when the scrap has no neighbour to draw. `inbound` and `outbound`
    /// are the same lists the connections sections render, so no extra walk
    /// over the wiki happens here.
    pub fn new(
        center_title: &str,
        inbound: &[Scrap],
        outbound: &[Scrap],
        limit: usize,
    ) -> Option<ScrapGraph> {
        let ins: HashSet<ScrapKey> = inbound.iter().map(|scrap| scrap.self_key()).collect();
        let outs: HashSet<ScrapKey> = outbound.iter().map(|scrap| scrap.self_key()).collect();

        let mut both: Vec<ScrapKey> = ins.intersection(&outs).cloned().collect();
        let mut only_in: Vec<ScrapKey> = ins.difference(&outs).cloned().collect();
        let mut only_out: Vec<ScrapKey> = outs.difference(&ins).cloned().collect();
        for group in [&mut both, &mut only_in, &mut only_out] {
            group.sort_by_key(sort_key);
        }

        let total = both.len() + only_in.len() + only_out.len();
        if total == 0 {
            return None;
        }
        let selected = Self::select(&both, &only_in, &only_out, limit);

        let mut nodes: Vec<GraphNode> = selected
            .into_iter()
            .map(|(key, dir)| GraphNode {
                label: label_of(&key.title().to_string()),
                key,
                dir,
                x: 0.0,
                y: 0.0,
            })
            .collect();
        Self::place(&mut nodes);
        Self::declutter(&mut nodes);

        // Padding follows the longest label rather than a fixed margin: a
        // clipped title is worse than a wide drawing, and short titles keep
        // the graph compact.
        let center_label = label_of(center_title);
        let pad_x = nodes
            .iter()
            .map(|node| &node.label)
            .chain(std::iter::once(&center_label))
            .fold(MIN_PAD_X, |m, label| {
                m.max(label_width(label) + LABEL_OFFSET * 2.0)
            });
        let ext_x = nodes.iter().fold(90.0_f32, |m, n| m.max(n.x.abs()));
        let ext_y = nodes.iter().fold(70.0_f32, |m, n| m.max(n.y.abs()));
        let width = (ext_x + pad_x) * 2.0;
        let height = (ext_y + PAD_Y) * 2.0;
        let (center_x, center_y) = (width / 2.0, height / 2.0);
        for node in nodes.iter_mut() {
            node.x += center_x;
            node.y += center_y;
        }

        Some(ScrapGraph {
            dropped: total - nodes.len(),
            nodes,
            center_label,
            width,
            height,
            center_x,
            center_y,
        })
    }

    /// Truncating one sorted list would let a hub's backlinks crowd out the
    /// handful of outgoing links entirely, so the cap is spent round-robin and
    /// every direction that exists keeps a seat.
    fn select(
        both: &[ScrapKey],
        only_in: &[ScrapKey],
        only_out: &[ScrapKey],
        limit: usize,
    ) -> Vec<(ScrapKey, LinkDir)> {
        let groups = [
            (both, LinkDir::Both),
            (only_in, LinkDir::In),
            (only_out, LinkDir::Out),
        ];
        let mut taken = [0usize; 3];
        let mut out: Vec<(ScrapKey, LinkDir)> = Vec::new();

        while out.len() < limit {
            let mut progressed = false;
            for (i, (group, dir)) in groups.iter().enumerate() {
                if out.len() >= limit {
                    break;
                }
                if let Some(key) = group.get(taken[i]) {
                    taken[i] += 1;
                    out.push((key.clone(), *dir));
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
        out.sort_by_key(|entry| sort_key(&entry.0));
        out
    }

    /// Backlinks on the left arc, links on the right, mutual on the vertical.
    /// A hub hangs a dozen neighbours off one side, so the radius grows out of
    /// the count rather than staying fixed.
    fn place(nodes: &mut [GraphNode]) {
        for (dir, base, spread) in [
            (LinkDir::In, PI, SPREAD_SIDE),
            (LinkDir::Out, 0.0, SPREAD_SIDE),
            (LinkDir::Both, PI * 1.5, SPREAD_BOTH),
        ] {
            let idx: Vec<usize> = (0..nodes.len()).filter(|&i| nodes[i].dir == dir).collect();
            let count = idx.len() as f32;
            let radius = BASE_RADIUS.max(count * NODE_GAP / spread);
            for (k, &i) in idx.iter().enumerate() {
                let t = if count <= 1.0 {
                    0.5
                } else {
                    k as f32 / (count - 1.0)
                };
                let angle = base - spread / 2.0 + spread * t;
                nodes[i].x = radius * angle.cos();
                nodes[i].y = radius * angle.sin();
            }
        }
    }

    /// Even spacing along an arc still lets two labels on the same side land at
    /// the same height. Push them apart vertically; the side they sit on — the
    /// part that carries the direction — does not move.
    fn declutter(nodes: &mut [GraphNode]) {
        for left in [true, false] {
            let mut idx: Vec<usize> = (0..nodes.len())
                .filter(|&i| (nodes[i].x < 0.0) == left)
                .collect();
            idx.sort_by(|&a, &b| nodes[a].y.total_cmp(&nodes[b].y));
            let before: f32 = idx.iter().map(|&i| nodes[i].y).sum();
            for k in 1..idx.len() {
                let prev = nodes[idx[k - 1]].y;
                if nodes[idx[k]].y - prev < LABEL_GAP {
                    nodes[idx[k]].y = prev + LABEL_GAP;
                }
            }
            // The pass only ever pushes down, so the whole side would sink
            // below the centre. Give back the drift it introduced.
            if !idx.is_empty() {
                let after: f32 = idx.iter().map(|&i| nodes[i].y).sum();
                let drift = (after - before) / idx.len() as f32;
                for &i in &idx {
                    nodes[i].y -= drift;
                }
            }
        }
    }
}

impl GraphNode {
    /// Where the edge meets the centre node, clear of its circle.
    pub fn edge_start(&self, center_x: f32, center_y: f32) -> (f32, f32) {
        let (ux, uy) = self.unit(center_x, center_y);
        (center_x + ux * CENTER_R, center_y + uy * CENTER_R)
    }

    /// Where the edge stops short of this node, leaving room for the arrowhead.
    pub fn edge_end(&self, center_x: f32, center_y: f32) -> (f32, f32) {
        let (ux, uy) = self.unit(center_x, center_y);
        let back = NODE_R + ARROW_CLEARANCE;
        (self.x - ux * back, self.y - uy * back)
    }

    /// Every label hangs off one side or the other. A centred label would sit
    /// above its node and spread both ways, which is the one shape the
    /// vertical declutter pass cannot keep clear of its neighbours.
    pub fn label_anchor(&self, center_x: f32) -> &'static str {
        if self.x < center_x {
            "end"
        } else {
            "start"
        }
    }

    pub fn label_x(&self, center_x: f32) -> f32 {
        if self.label_anchor(center_x) == "end" {
            self.x - LABEL_OFFSET
        } else {
            self.x + LABEL_OFFSET
        }
    }

    pub fn label_y(&self) -> f32 {
        self.y + 4.0
    }

    pub fn arrow_at_center(&self) -> bool {
        matches!(self.dir, LinkDir::In | LinkDir::Both)
    }

    pub fn arrow_at_node(&self) -> bool {
        matches!(self.dir, LinkDir::Out | LinkDir::Both)
    }

    fn unit(&self, center_x: f32, center_y: f32) -> (f32, f32) {
        let (dx, dy) = (self.x - center_x, self.y - center_y);
        let distance = (dx * dx + dy * dy).sqrt().max(0.001);
        (dx / distance, dy / distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scrap(title: &str, text: &str) -> Scrap {
        Scrap::new(title, &None, text)
    }

    fn dir_of(graph: &ScrapGraph, title: &str) -> LinkDir {
        graph
            .nodes
            .iter()
            .find(|node| node.key.title().to_string() == title)
            .unwrap_or_else(|| panic!("{title} not in graph"))
            .dir
    }

    #[test]
    fn it_returns_none_without_neighbors() {
        assert_eq!(ScrapGraph::new("center", &[], &[], MAX_NODES), None);
    }

    #[test]
    fn it_classifies_each_neighbor_by_direction() {
        let inbound = vec![scrap("in", ""), scrap("mutual", "")];
        let outbound = vec![scrap("out", ""), scrap("mutual", "")];

        let graph = ScrapGraph::new("center", &inbound, &outbound, MAX_NODES).unwrap();

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(dir_of(&graph, "in"), LinkDir::In);
        assert_eq!(dir_of(&graph, "out"), LinkDir::Out);
        assert_eq!(dir_of(&graph, "mutual"), LinkDir::Both);
        assert_eq!(graph.dropped, 0);
    }

    #[test]
    fn it_puts_backlinks_left_and_links_right() {
        let inbound = vec![scrap("in", "")];
        let outbound = vec![scrap("out", "")];

        let graph = ScrapGraph::new("center", &inbound, &outbound, MAX_NODES).unwrap();

        let node = |title: &str| {
            graph
                .nodes
                .iter()
                .find(|n| n.key.title().to_string() == title)
                .unwrap()
                .clone()
        };
        assert!(node("in").x < graph.center_x);
        assert!(node("out").x > graph.center_x);
    }

    #[test]
    fn it_keeps_both_directions_when_the_cap_bites() {
        // A hub: many backlinks, one outgoing link. Alphabetically the link
        // sorts last, so a plain truncation would drop it.
        let inbound: Vec<Scrap> = (0..30).map(|i| scrap(&format!("a{i:02}"), "")).collect();
        let outbound = vec![scrap("zzz", "")];

        let graph = ScrapGraph::new("center", &inbound, &outbound, MAX_NODES).unwrap();

        assert_eq!(graph.nodes.len(), MAX_NODES);
        assert_eq!(graph.dropped, 31 - MAX_NODES);
        assert_eq!(dir_of(&graph, "zzz"), LinkDir::Out);
    }

    #[test]
    fn it_orders_nodes_independently_of_read_order() {
        let forward = vec![scrap("a", ""), scrap("b", ""), scrap("c", "")];
        let reversed = vec![scrap("c", ""), scrap("b", ""), scrap("a", "")];

        let one = ScrapGraph::new("center", &forward, &[], MAX_NODES).unwrap();
        let other = ScrapGraph::new("center", &reversed, &[], MAX_NODES).unwrap();

        assert_eq!(one, other);
    }

    #[test]
    fn it_separates_labels_that_share_a_height() {
        let inbound: Vec<Scrap> = (0..12).map(|i| scrap(&format!("in{i:02}"), "")).collect();

        let graph = ScrapGraph::new("center", &inbound, &[], MAX_NODES).unwrap();

        let mut ys: Vec<f32> = graph
            .nodes
            .iter()
            .filter(|n| n.x < graph.center_x)
            .map(|n| n.y)
            .collect();
        ys.sort_by(f32::total_cmp);
        for pair in ys.windows(2) {
            assert!(
                pair[1] - pair[0] >= LABEL_GAP - 0.01,
                "labels {} and {} are too close",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn it_sizes_the_viewbox_around_the_nodes() {
        let inbound: Vec<Scrap> = (0..16).map(|i| scrap(&format!("in{i:02}"), "")).collect();

        let graph = ScrapGraph::new("center", &inbound, &[], MAX_NODES).unwrap();

        for node in &graph.nodes {
            assert!(node.x > 0.0 && node.x < graph.width);
            assert!(node.y > 0.0 && node.y < graph.height);
        }
    }

    #[test]
    fn it_points_arrows_at_the_referenced_side() {
        let inbound = vec![scrap("in", ""), scrap("mutual", "")];
        let outbound = vec![scrap("out", ""), scrap("mutual", "")];

        let graph = ScrapGraph::new("center", &inbound, &outbound, MAX_NODES).unwrap();

        let node = |title: &str| {
            graph
                .nodes
                .iter()
                .find(|n| n.key.title().to_string() == title)
                .unwrap()
        };
        assert!(node("in").arrow_at_center() && !node("in").arrow_at_node());
        assert!(!node("out").arrow_at_center() && node("out").arrow_at_node());
        assert!(node("mutual").arrow_at_center() && node("mutual").arrow_at_node());
    }
}
