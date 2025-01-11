use scraps_libs::model::scrap::Scrap;

#[derive(Clone)]
pub enum Paging {
    Not,
    By(usize),
}

impl Paging {
    pub fn size_with(&self, scraps: &[Scrap]) -> usize {
        match self {
            Paging::Not => scraps.len(),
            Paging::By(size) => size.to_owned(),
        }
    }
}
