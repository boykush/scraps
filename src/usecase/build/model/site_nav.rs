use chrono_tz::Tz;
use scraps_libs::model::{scrap::Scrap, tags::Tags};

use super::{backlinks_map::BacklinksMap, scope_counts::ScopeCounts, scrap_types::ScrapTypes};

/// Everything the shell needs on every page: the scrap count, the tags a
/// reader can scope a list to with their counts per type, the types in use,
/// whether the search UI is wired up, and the timezone that commit dates
/// render in. Built once per build and shared by every render.
pub struct SiteNav {
    pub scrap_count: usize,
    pub tags: Tags,
    pub types: ScrapTypes,
    pub scope_counts: ScopeCounts,
    pub build_search_index: bool,
    pub timezone: Tz,
    /// A root README.md renders at its own /about/ page; the sidebar links
    /// it only when the file exists.
    pub has_readme: bool,
}

impl SiteNav {
    pub fn new(
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
        build_search_index: bool,
        timezone: Tz,
        has_readme: bool,
    ) -> SiteNav {
        SiteNav {
            scrap_count: scraps.len(),
            tags: Tags::new(scraps),
            types: ScrapTypes::new(scraps),
            scope_counts: ScopeCounts::new(scraps, backlinks_map),
            build_search_index,
            timezone,
            has_readme,
        }
    }
}
