use crate::{Build, View, Widget};

#[derive(Debug)]
pub struct Row {
    pub children: Vec<View>,
}

impl Widget for Row {
    fn build(&self, mut build: Build) {
        for view in &self.children {
            unsafe {
                build.add_view(view);
            }
        }
    }
}
