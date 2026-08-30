use scraps_libs::model::tags::Tags;

/// Everything the persistent sidebar needs on every page: the view nav's
/// scrap count, the tag list, and whether the search UI is wired up.
/// Built once per build and shared by every page render.
pub struct SiteNav {
    pub scrap_count: usize,
    pub tags: Tags,
    pub build_search_index: bool,
}

impl SiteNav {
    pub fn new(scrap_count: usize, tags: Tags, build_search_index: bool) -> SiteNav {
        SiteNav {
            scrap_count,
            tags,
            build_search_index,
        }
    }
}
