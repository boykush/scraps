use chrono_tz::Tz;
use scraps_libs::model::tags::Tags;

/// Everything the shell needs on every page: the view nav's scrap count,
/// the tag list, whether the search UI is wired up, and the timezone that
/// commit dates render in. Built once per build and shared by every render.
pub struct SiteNav {
    pub scrap_count: usize,
    pub tags: Tags,
    pub build_search_index: bool,
    pub timezone: Tz,
}

impl SiteNav {
    pub fn new(scrap_count: usize, tags: Tags, build_search_index: bool, timezone: Tz) -> SiteNav {
        SiteNav {
            scrap_count,
            tags,
            build_search_index,
            timezone,
        }
    }
}
