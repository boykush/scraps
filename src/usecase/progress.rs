#[derive(Debug)]
pub enum Stage {
    ReadScraps,
    GenerateHtml,
    GenerateCss,
    GenerateJson,
}

pub trait Progress {
    fn start_stage(&self, stage: &Stage);
    fn complete_stage(&self, stage: &Stage, count: &usize);
    fn end(&self);
}

#[cfg(test)]
pub mod tests {
    use super::{Progress, Stage};

    pub struct ProgressTest();
    impl ProgressTest {
        pub fn new() -> Self {
            Self {}
        }
    }
    impl Progress for ProgressTest {
        fn start_stage(&self, _stage: &Stage) {
            println!("Start");
        }

        fn complete_stage(&self, _stage: &Stage, _count: &usize) {
            println!("Complete");
        }

        fn end(&self) {
            println!("End");
        }
    }
}
