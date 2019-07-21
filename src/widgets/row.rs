use crate::{Build, Layout, Layouter, View, Widget};
use stretch::style::{Dimension, Style};

#[derive(Debug)]
pub struct Row {
    pub children: Vec<View>,
}

impl Widget for Row {
    fn build(&self, mut build: Build) {
        for view in &self.children {
            unsafe { build.clone().add_view(view) };
        }
    }

    fn layout(&self, mut layout: Layouter) -> Layout {
        Layout::Flex(Style {
            size: stretch::geometry::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
            ..Style::default()
        })
    }
}
