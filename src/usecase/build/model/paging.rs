#[derive(Clone)]
pub enum Paging {
    Not,
    By(usize),
}

impl Paging {
    pub fn size_with(&self, scrap_count: usize) -> usize {
        match self {
            Paging::Not => scrap_count,
            Paging::By(size) => size.to_owned(),
        }
    }
}
