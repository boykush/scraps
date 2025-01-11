use super::{paging::Paging, sort::SortKey};

pub struct ListViewConfigs {
    pub build_search_index: bool,
    pub sort_key: SortKey,
    pub paging: Paging,
}

impl ListViewConfigs {
    pub fn new(build_search_index: &bool, sort_key: &SortKey, paging: &Paging) -> ListViewConfigs {
        ListViewConfigs {
            build_search_index: *build_search_index,
            sort_key: sort_key.clone(),
            paging: paging.clone(),
        }
    }
}
