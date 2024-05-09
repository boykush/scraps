use super::scrap::Scrap;

pub enum Paging {
    Not,
    By(usize)
}

impl Paging {
    pub fn size_with(&self, scraps: &Vec<Scrap>) -> usize {
        match self {
            Paging::Not => scraps.len(),
            Paging::By(size) => size.to_owned()
        }
    }
}