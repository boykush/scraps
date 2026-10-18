use std::collections::BTreeMap;

use scraps_libs::model::{page_path::PagePath, tag::Tag};

use crate::usecase::build::model::{scope_counts::TypeCounts, site_nav::SiteNav};

/// One scope in the sidebar: the whole wiki, or a tag.
#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct NavScopeTera {
    /// Full tag path, e.g. `ai/ml`; empty for the whole wiki.
    title: String,
    /// What the sidebar shows: the tag's own segment, nested under its parent.
    name: String,
    /// How far the entry is indented: 0 for the whole wiki and root tags.
    depth: usize,
    /// Page URL relative to `base_url`, already percent-encoded.
    url_path: String,
    counts: TypeCounts,
    active: bool,
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct NavScopesTera(Vec<NavScopeTera>);

impl NavScopesTera {
    /// The whole wiki first, then every tag as a tree: siblings most used
    /// first, the name breaking ties so the order is the same on every build.
    /// `active` is the tag a list page is scoped to; a page that lists
    /// nothing marks no entry.
    pub fn new(site_nav: &SiteNav, active: Option<&Tag>, is_list: bool) -> NavScopesTera {
        let counts = &site_nav.scope_counts;
        let mut children: BTreeMap<Option<Tag>, Vec<&Tag>> = BTreeMap::new();
        for tag in site_nav.tags.iter() {
            let parent = tag.ancestors().pop();
            children.entry(parent).or_default().push(tag);
        }
        for siblings in children.values_mut() {
            siblings.sort_by(|a, b| {
                let (a_total, b_total) = (counts.of(Some(a)).total, counts.of(Some(b)).total);
                b_total.cmp(&a_total).then_with(|| a.cmp(b))
            });
        }

        let mut scopes = vec![NavScopeTera {
            title: String::new(),
            name: "all scraps".to_string(),
            depth: 0,
            url_path: String::new(),
            counts: counts.of(None).clone(),
            active: is_list && active.is_none(),
        }];
        let mut stack: Vec<&Tag> = children.get(&None).cloned().unwrap_or_default();
        stack.reverse();
        while let Some(tag) = stack.pop() {
            scopes.push(NavScopeTera {
                title: tag.to_string(),
                name: tag.name().to_string(),
                depth: tag.depth() - 1,
                url_path: PagePath::of_tag(tag).to_url_path(),
                counts: counts.of(Some(tag)).clone(),
                active: is_list && active == Some(tag),
            });
            if let Some(kids) = children.get(&Some(tag.clone())) {
                stack.extend(kids.iter().rev());
            }
        }
        NavScopesTera(scopes)
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::model::scrap::Scrap;

    use super::*;
    use crate::usecase::build::model::backlinks_map::BacklinksMap;

    fn nav(
        scraps: &[Scrap],
        active: Option<&Tag>,
        is_list: bool,
    ) -> Vec<(String, usize, usize, bool)> {
        let site_nav = SiteNav::new(
            scraps,
            &BacklinksMap::new(scraps),
            false,
            chrono_tz::UTC,
            false,
        );
        NavScopesTera::new(&site_nav, active, is_list)
            .0
            .into_iter()
            .map(|s| (s.name, s.depth, s.counts.total, s.active))
            .collect()
    }

    #[test]
    fn it_lists_the_whole_wiki_then_tags_as_a_tree() {
        let scraps = [
            Scrap::new("a", &None, "#[[Security/Auth]] #[[Agile]]"),
            Scrap::new("b", &None, "#[[Security/Crypto]]"),
            Scrap::new("c", &None, "#[[Security/Crypto]]"),
            Scrap::new("d", &None, "#[[Agile]]"),
        ];

        assert_eq!(
            nav(&scraps, None, true),
            vec![
                ("all scraps".to_string(), 0, 4, true),
                ("Security".to_string(), 0, 3, false),
                ("Crypto".to_string(), 1, 2, false),
                ("Auth".to_string(), 1, 1, false),
                ("Agile".to_string(), 0, 2, false),
            ]
        );
    }

    #[test]
    fn it_marks_the_scope_a_page_lists() {
        let scraps = [Scrap::new("a", &None, "#[[ai/ml]]")];
        let ai_ml = Tag::from("ai/ml");
        let marked = |active, is_list| -> Vec<bool> {
            nav(&scraps, active, is_list)
                .into_iter()
                .map(|(_, _, _, active)| active)
                .collect()
        };
        assert_eq!(marked(Some(&ai_ml), true), vec![false, false, true]);
        assert_eq!(marked(None, false), vec![false, false, false]);
    }
}
