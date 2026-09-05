use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::ScrapsResult;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;

// A hub scrap pulls in hundreds of neighbors within three hops, so depth is
// only ever a knob: the node cap is what actually bounds the response.
pub const MAX_DEPTH: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapRef {
    pub title: Title,
    pub ctx: Option<Ctx>,
}

impl From<&ScrapKey> for ScrapRef {
    fn from(key: &ScrapKey) -> Self {
        ScrapRef {
            title: key.title().clone(),
            ctx: key.ctx().clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborhoodNode {
    pub scrap: ScrapRef,
    pub hop: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborhoodEdge {
    pub from: ScrapRef,
    pub to: ScrapRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupScrapNeighborhoodResult {
    pub nodes: Vec<NeighborhoodNode>,
    pub edges: Vec<NeighborhoodEdge>,
    /// Neighbours the node cap kept out; raising the cap brings them back.
    pub dropped: usize,
}

fn sort_key(key: &ScrapKey) -> (String, String) {
    (
        key.ctx()
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default(),
        key.title().to_string(),
    )
}

pub struct LookupScrapNeighborhoodUsecase;

impl LookupScrapNeighborhoodUsecase {
    pub fn new() -> LookupScrapNeighborhoodUsecase {
        LookupScrapNeighborhoodUsecase
    }

    pub fn execute(
        &self,
        scraps: &[Scrap],
        title: &Title,
        ctx: &Option<Ctx>,
        depth: usize,
        limit: usize,
    ) -> ScrapsResult<LookupScrapNeighborhoodResult> {
        let depth = depth.min(MAX_DEPTH);
        let root = ScrapKey::new(title, ctx);

        let by_key: HashMap<ScrapKey, &Scrap> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();
        if !by_key.contains_key(&root) {
            return Err(anyhow::anyhow!(
                "Scrap not found: title='{}', ctx='{:?}'",
                title,
                ctx
            ));
        }
        let backlinks = BacklinksMap::new(scraps);

        let mut hops: HashMap<ScrapKey, usize> = HashMap::new();
        let mut order: Vec<ScrapKey> = Vec::new();
        let mut dropped: HashSet<ScrapKey> = HashSet::new();
        let mut queue: VecDeque<ScrapKey> = VecDeque::new();

        if Self::admit(&root, 0, limit, &mut hops, &mut order, &mut dropped) {
            queue.push_back(root);
        }

        while let Some(key) = queue.pop_front() {
            let hop = hops[&key];
            if hop >= depth {
                continue;
            }
            for neighbor in Self::neighbors(&key, &by_key, &backlinks) {
                if Self::admit(
                    &neighbor,
                    hop + 1,
                    limit,
                    &mut hops,
                    &mut order,
                    &mut dropped,
                ) {
                    queue.push_back(neighbor);
                }
            }
        }

        let nodes = order
            .iter()
            .map(|key| NeighborhoodNode {
                scrap: key.into(),
                hop: hops[key],
            })
            .collect();

        Ok(LookupScrapNeighborhoodResult {
            nodes,
            edges: Self::induced_edges(&order, &hops, &by_key),
            dropped: dropped.len(),
        })
    }

    fn admit(
        key: &ScrapKey,
        hop: usize,
        limit: usize,
        hops: &mut HashMap<ScrapKey, usize>,
        order: &mut Vec<ScrapKey>,
        dropped: &mut HashSet<ScrapKey>,
    ) -> bool {
        if hops.contains_key(key) {
            return false;
        }
        if hops.len() >= limit {
            dropped.insert(key.clone());
            return false;
        }
        hops.insert(key.clone(), hop);
        order.push(key.clone());
        true
    }

    fn neighbors(
        key: &ScrapKey,
        by_key: &HashMap<ScrapKey, &Scrap>,
        backlinks: &BacklinksMap,
    ) -> Vec<ScrapKey> {
        let mut seen: HashSet<ScrapKey> = HashSet::new();
        let mut neighbors: Vec<ScrapKey> = Vec::new();

        // A link to a scrap that does not exist yet has nothing to walk to.
        if let Some(scrap) = by_key.get(key) {
            for link in scrap.links() {
                if by_key.contains_key(link) && seen.insert(link.clone()) {
                    neighbors.push(link.clone());
                }
            }
        }
        // Backlinks arrive in whatever order the wiki was read in, which would
        // make the same call return a different map — and, under the cap, a
        // different set of scraps. Name order pins it.
        let mut inbound: Vec<ScrapKey> = backlinks
            .get(key)
            .iter()
            .map(|linking| linking.self_key())
            .filter(|linking_key| seen.insert(linking_key.clone()))
            .collect();
        inbound.sort_by_key(sort_key);
        neighbors.extend(inbound);

        neighbors
    }

    // Every link between two returned nodes, not just the ones the walk came in
    // on: the map is worth more than the path taken through it.
    fn induced_edges(
        order: &[ScrapKey],
        hops: &HashMap<ScrapKey, usize>,
        by_key: &HashMap<ScrapKey, &Scrap>,
    ) -> Vec<NeighborhoodEdge> {
        let mut seen: HashSet<(ScrapKey, ScrapKey)> = HashSet::new();
        let mut edges: Vec<NeighborhoodEdge> = Vec::new();

        for key in order {
            let Some(scrap) = by_key.get(key) else {
                continue;
            };
            for link in scrap.links() {
                if !hops.contains_key(link) {
                    continue;
                }
                if seen.insert((key.clone(), link.clone())) {
                    edges.push(NeighborhoodEdge {
                        from: key.into(),
                        to: link.into(),
                    });
                }
            }
        }

        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scrap(title: &str, text: &str) -> Scrap {
        Scrap::new(title, &None, text)
    }

    fn titles(nodes: &[NeighborhoodNode]) -> Vec<String> {
        nodes.iter().map(|n| n.scrap.title.to_string()).collect()
    }

    fn hop_of(result: &LookupScrapNeighborhoodResult, title: &str) -> usize {
        result
            .nodes
            .iter()
            .find(|n| n.scrap.title.to_string() == title)
            .unwrap_or_else(|| panic!("{title} should be a node: {:?}", titles(&result.nodes)))
            .hop
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-01
    #[test]
    fn test_one_call_returns_the_neighborhood_and_its_edges() {
        let scraps = vec![
            scrap("microservices", "# microservices\n\nlinks to [[ddd]]"),
            scrap("ddd", "# ddd\n\ncontent"),
            scrap("monolith", "# monolith\n\nlinks to [[microservices]]"),
        ];

        let result = LookupScrapNeighborhoodUsecase::new()
            .execute(&scraps, &Title::from("microservices"), &None, 1, 50)
            .unwrap();

        assert_eq!(
            titles(&result.nodes),
            vec!["microservices", "ddd", "monolith"]
        );
        assert_eq!(hop_of(&result, "microservices"), 0);
        assert_eq!(result.edges.len(), 2);
        assert_eq!(result.dropped, 0);
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-03
    #[test]
    fn test_the_given_scrap_is_the_only_root() {
        let scraps = vec![
            scrap("rust guide", "# rust guide\n\n[[rust patterns]]"),
            scrap("rust patterns", "# rust patterns\n\ncontent"),
            scrap("rust tooling", "# rust tooling\n\ncontent"),
        ];

        let result = LookupScrapNeighborhoodUsecase::new()
            .execute(&scraps, &Title::from("rust guide"), &None, 1, 50)
            .unwrap();

        assert_eq!(titles(&result.nodes), vec!["rust guide", "rust patterns"]);
        assert_eq!(
            result.nodes.iter().filter(|n| n.hop == 0).count(),
            1,
            "only the given scrap sits at hop 0"
        );
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-03
    #[test]
    fn test_an_unknown_scrap_is_an_error_not_an_empty_map() {
        let scraps = vec![scrap("known", "# known\n\ncontent")];

        let result = LookupScrapNeighborhoodUsecase::new().execute(
            &scraps,
            &Title::from("unknown"),
            &None,
            1,
            50,
        );

        assert!(result.is_err());
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-04
    #[test]
    fn test_edges_are_wiki_links_in_both_directions_only() {
        let scraps = vec![
            scrap("root", "# root\n\n[[outbound]] #[[shared]]"),
            scrap("outbound", "# outbound\n\ncontent"),
            scrap("inbound", "# inbound\n\n[[root]]"),
            scrap("tag mate", "# tag mate\n\n#[[shared]]"),
        ];

        let result = LookupScrapNeighborhoodUsecase::new()
            .execute(&scraps, &Title::from("root"), &None, 1, 50)
            .unwrap();

        assert_eq!(titles(&result.nodes), vec!["root", "outbound", "inbound"]);
        assert_eq!(hop_of(&result, "outbound"), 1);
        assert_eq!(hop_of(&result, "inbound"), 1);

        let edges: Vec<(String, String)> = result
            .edges
            .iter()
            .map(|e| (e.from.title.to_string(), e.to.title.to_string()))
            .collect();
        assert!(edges.contains(&("root".to_string(), "outbound".to_string())));
        assert!(edges.contains(&("inbound".to_string(), "root".to_string())));
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-04
    #[test]
    fn test_a_scrap_reached_twice_keeps_its_shortest_hop() {
        let scraps = vec![
            scrap("root", "# root\n\n[[near]] [[far]]"),
            scrap("near", "# near\n\n[[far]]"),
            scrap("far", "# far\n\ncontent"),
        ];

        let result = LookupScrapNeighborhoodUsecase::new()
            .execute(&scraps, &Title::from("root"), &None, 2, 50)
            .unwrap();

        assert_eq!(result.nodes.len(), 3);
        assert_eq!(hop_of(&result, "far"), 1);
        assert_eq!(result.edges.len(), 3);
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-05
    #[test]
    fn test_depth_opens_one_ring_at_a_time_and_stops_at_five() {
        let mut scraps = vec![scrap("s0", "# s0\n\n[[s1]]")];
        for i in 1..=6 {
            scraps.push(scrap(
                &format!("s{i}"),
                &format!("# s{i}\n\n[[s{}]]", i + 1),
            ));
        }
        scraps.push(scrap("s7", "# s7\n\ncontent"));
        let usecase = LookupScrapNeighborhoodUsecase::new();
        let root = Title::from("s0");

        let shallow = usecase.execute(&scraps, &root, &None, 1, 50).unwrap();
        assert_eq!(titles(&shallow.nodes), vec!["s0", "s1"]);

        let deeper = usecase.execute(&scraps, &root, &None, 2, 50).unwrap();
        assert_eq!(titles(&deeper.nodes), vec!["s0", "s1", "s2"]);

        let over = usecase.execute(&scraps, &root, &None, 9, 50).unwrap();
        assert_eq!(over.nodes.len(), MAX_DEPTH + 1);
        assert_eq!(hop_of(&over, "s5"), MAX_DEPTH);
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-06
    #[test]
    fn test_the_same_call_returns_the_same_map() {
        let linkers: Vec<Scrap> = ["c", "a", "b"]
            .iter()
            .map(|t| scrap(t, &format!("# {t}\n\n[[root]]")))
            .collect();
        let mut scraps = vec![scrap("root", "# root\n\ncontent")];
        scraps.extend(linkers);
        let mut reversed = scraps.clone();
        reversed.reverse();
        let usecase = LookupScrapNeighborhoodUsecase::new();

        let read_one = usecase
            .execute(&scraps, &Title::from("root"), &None, 1, 50)
            .unwrap();
        let read_other = usecase
            .execute(&reversed, &Title::from("root"), &None, 1, 50)
            .unwrap();

        assert_eq!(titles(&read_one.nodes), vec!["root", "a", "b", "c"]);
        assert_eq!(titles(&read_other.nodes), titles(&read_one.nodes));
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-06
    #[test]
    fn test_node_cap_keeps_the_near_and_counts_what_it_dropped() {
        let hub_links: String = (1..=20).map(|i| format!("[[n{i:02}]] ")).collect();
        let mut scraps = vec![scrap("hub", &format!("# hub\n\n{hub_links}"))];
        for i in 1..=20 {
            scraps.push(scrap(&format!("n{i:02}"), "content"));
        }

        let result = LookupScrapNeighborhoodUsecase::new()
            .execute(&scraps, &Title::from("hub"), &None, 2, 5)
            .unwrap();

        assert_eq!(result.nodes.len(), 5);
        assert_eq!(hop_of(&result, "hub"), 0);
        assert_eq!(result.dropped, 16);
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-01
    #[test]
    fn test_a_scrap_with_no_relations_returns_itself_alone() {
        let scraps = vec![
            scrap("lonely", "# lonely\n\ncontent"),
            scrap("elsewhere", "# elsewhere\n\ncontent"),
        ];

        let result = LookupScrapNeighborhoodUsecase::new()
            .execute(&scraps, &Title::from("lonely"), &None, 2, 50)
            .unwrap();

        assert_eq!(titles(&result.nodes), vec!["lonely"]);
        assert!(result.edges.is_empty());
    }
}
