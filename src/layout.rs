use crate::storage::Storage;
use crate::WidgetId;

#[derive(Debug, Clone)]
pub enum Layout {
    Flex(stretch::style::Style),
    Pass,
}

pub enum LayoutNode {
    FlexRoot(stretch::node::Node),
    Flex { node: stretch::node::Node, root: stretch::node::Node },
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum ParentLayout {
    Node { node: stretch::node::Node, root: stretch::node::Node },
}

pub type Layouts = Storage<LayoutNode>;

impl std::ops::Index<WidgetId> for Layouts {
    type Output = LayoutNode;
    fn index(&self, idx: WidgetId) -> &Self::Output {
        self.get(idx.0)
    }
}

impl std::ops::IndexMut<WidgetId> for Layouts {
    fn index_mut(&mut self, idx: WidgetId) -> &mut Self::Output {
        self.get_mut(idx.0)
    }
}
