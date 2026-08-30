use super::paging::Paging;

pub struct ListViewConfigs {
    pub build_search_index: bool,
    pub paging: Paging,
}

impl ListViewConfigs {
    pub fn new(build_search_index: &bool, paging: &Paging) -> ListViewConfigs {
        ListViewConfigs {
            build_search_index: *build_search_index,
            paging: paging.clone(),
        }
    }
}
