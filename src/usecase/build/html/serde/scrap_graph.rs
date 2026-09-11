use scraps_libs::model::file::ScrapFileStem;

use crate::usecase::build::model::scrap_graph::{GraphNode, LinkDir, ScrapGraph, NODE_R};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeGraphNode {
    ctx: Option<String>,
    title: String,
    label: String,
    html_file_name: String,
    /// "in" | "out" | "both" — a class hook so a user template can style the
    /// directions without re-deriving them.
    dir: &'static str,
    x: f32,
    y: f32,
    r: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    label_x: f32,
    label_y: f32,
    anchor: &'static str,
    arrow_start: bool,
    arrow_end: bool,
}

impl SerializeGraphNode {
    fn new(node: &GraphNode, center_x: f32, center_y: f32) -> SerializeGraphNode {
        let (x1, y1) = node.edge_start(center_x, center_y);
        let (x2, y2) = node.edge_end(center_x, center_y);
        let title = node.key.title().to_string();
        SerializeGraphNode {
            ctx: node.key.ctx().as_ref().map(|ctx| ctx.to_string()),
            label: node.label.clone(),
            title,
            html_file_name: format!("{}.html", ScrapFileStem::from(node.key.clone())),
            dir: match node.dir {
                LinkDir::In => "in",
                LinkDir::Out => "out",
                LinkDir::Both => "both",
            },
            x: node.x,
            y: node.y,
            r: NODE_R,
            x1,
            y1,
            x2,
            y2,
            label_x: node.label_x(center_x),
            label_y: node.label_y(),
            anchor: node.label_anchor(center_x),
            arrow_start: node.arrow_at_center(),
            arrow_end: node.arrow_at_node(),
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct ScrapGraphTera {
    width: f32,
    height: f32,
    center_x: f32,
    center_y: f32,
    center_r: f32,
    center_label: String,
    center_label_y: f32,
    more_x: f32,
    more_y: f32,
    dropped: usize,
    nodes: Vec<SerializeGraphNode>,
}

impl From<&ScrapGraph> for ScrapGraphTera {
    fn from(graph: &ScrapGraph) -> Self {
        ScrapGraphTera {
            width: graph.width,
            height: graph.height,
            center_x: graph.center_x,
            center_y: graph.center_y,
            center_r: graph.center_r(),
            center_label: graph.center_label.clone(),
            center_label_y: graph.center_label_y(),
            more_x: graph.width - 10.0,
            more_y: graph.height - 10.0,
            dropped: graph.dropped,
            nodes: graph
                .nodes
                .iter()
                .map(|node| SerializeGraphNode::new(node, graph.center_x, graph.center_y))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::model::{context::Ctx, scrap::Scrap};

    use crate::usecase::build::model::scrap_graph::MAX_NODES;

    use super::*;

    #[test]
    fn it_serializes_direction_and_link_target() {
        let inbound = vec![Scrap::new("in", &Some(Ctx::from("guide")), "")];
        let outbound = vec![Scrap::new("out", &None, "")];
        let graph = ScrapGraph::new("center", &inbound, &outbound, MAX_NODES).unwrap();

        let tera = ScrapGraphTera::from(&graph);

        let backlink = tera.nodes.iter().find(|n| n.title == "in").unwrap();
        assert_eq!(backlink.dir, "in");
        assert_eq!(backlink.ctx, Some("guide".to_string()));
        assert_eq!(backlink.html_file_name, "guide/in.html");
        assert!(backlink.arrow_start && !backlink.arrow_end);
        assert_eq!(backlink.anchor, "end");

        let link = tera.nodes.iter().find(|n| n.title == "out").unwrap();
        assert_eq!(link.dir, "out");
        assert!(!link.arrow_start && link.arrow_end);
        assert_eq!(link.anchor, "start");
    }

    #[test]
    fn it_stops_edges_short_of_both_endpoints() {
        let outbound = vec![Scrap::new("out", &None, "")];
        let graph = ScrapGraph::new("center", &[], &outbound, MAX_NODES).unwrap();

        let tera = ScrapGraphTera::from(&graph);

        let node = &tera.nodes[0];
        let from_center =
            ((node.x1 - tera.center_x).powi(2) + (node.y1 - tera.center_y).powi(2)).sqrt();
        assert!(from_center > 0.0, "edge starts away from the centre point");
        let to_node = ((node.x - node.x2).powi(2) + (node.y - node.y2).powi(2)).sqrt();
        assert!(to_node > 0.0, "edge stops short of the node circle");
    }

    #[test]
    fn it_shortens_a_long_title_but_keeps_it_for_the_tooltip() {
        let long = "a very long scrap title that will not fit on one node";
        let graph =
            ScrapGraph::new("center", &[], &[Scrap::new(long, &None, "")], MAX_NODES).unwrap();

        let tera = ScrapGraphTera::from(&graph);

        assert_eq!(tera.nodes[0].title, long);
        assert_eq!(tera.nodes[0].label, "a very long scra…");
    }
}
