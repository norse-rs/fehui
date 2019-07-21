#[macro_use]
extern crate derivative;

mod layout;
mod storage;
mod view;
pub mod widgets;

pub use layout::Layout;
pub use view::View;

use layout::{LayoutNode, Layouts, ParentLayout};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;
use storage::{Keys, Widgets};
use stretch::geometry::Size;
use stretch::number::ToNumber;
use stretch::Stretch;

pub type NodeId = indextree::NodeId; // temp
type Graph = indextree::Arena<WidgetId>;
type GraphMap = HashMap<WidgetId, NodeId>; // Map: Instance -> Graph

pub struct FehUI {
    pub graph: Graph, // temp pub
    graph_map: GraphMap,
    widgets: Widgets,
    widget_ids: WidgetIdGen,
    pub root: Option<Id>, // temp pub
    keys: Keys,
    states: StateCache,
    layouts: Layouts,
    stretch: Stretch,
}

impl FehUI {
    pub fn new() -> Self {
        FehUI {
            graph: Graph::new(),
            graph_map: GraphMap::new(),
            widgets: Widgets::new(),
            root: None,
            widget_ids: WidgetIdGen::new(),
            keys: Keys::new(),
            states: StateCache::new(),
            layouts: Layouts::new(),
            stretch: Stretch::new(),
        }
    }

    pub fn build(&mut self, widget: impl Widget) {
        self.widget_ids = WidgetIdGen::new();
        self.graph = Graph::new();

        let root = {
            let widget = self.widget_ids.generate();
            let node = self.graph.new_node(widget);
            Id { widget, node }
        };

        self.widgets.clear();
        self.graph_map.clear();
        self.keys.clear();
        self.layouts.clear();

        self.root = Some(root);

        self.keys.push(Some(GlobalKeygen::ROOT_KEY));
        self.widgets.push(Box::new(widget));
        self.graph_map.insert(root.widget, root.node);
        self.layouts.push(LayoutNode::None);

        let build = Build {
            id: root,
            widgets: &self.widgets,
            graph: &mut self.graph,
            graph_map: &mut self.graph_map,
            widget_ids: &mut self.widget_ids,
            keys: &mut self.keys,
            layouts: &mut self.layouts,
            states: &mut self.states,
        };

        // println!("build :: build {:?}", root);
        self.widgets[root.widget].build(build);
    }

    pub fn layout(&mut self, width: f32, height: f32) {
        self.generate_layout();
        self.compute_layout(width, height);
    }

    fn generate_layout(&mut self) {
        if let Some(root) = self.root {
            Layouter {
                id: root,
                widgets: &self.widgets,
                graph: &self.graph,
                layouts: &mut self.layouts,
                stretch: &mut self.stretch,
                parent: None,
            }
            .generate_tree();
        }
    }

    fn compute_layout(&mut self, width: f32, height: f32) {
        if let Some(root) = self.root {
            Layouter {
                id: root,
                widgets: &self.widgets,
                graph: &self.graph,
                layouts: &mut self.layouts,
                stretch: &mut self.stretch,
                parent: None,
            }
            .compute_layout(width, height);
        }
    }
}

pub trait Widget: 'static + std::fmt::Debug {
    fn build(&self, ctxt: Build);
    fn layout(&self, ctxt: Layouter) -> Layout;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Global(usize),
    Local { parent: WidgetId, local: usize },
}

#[derive(Debug)]
struct GlobalKeygen {
    next: usize,
}
impl GlobalKeygen {
    const ROOT_KEY: Key = Key::Global(0);

    pub fn new() -> Self {
        GlobalKeygen { next: 1 }
    }

    pub fn generate(&mut self) -> Key {
        let id = self.next;
        self.next += 1;
        Key::Global(id)
    }
}

pub struct Layouter<'a> {
    id: Id,
    widgets: &'a Widgets,
    graph: &'a Graph,
    layouts: &'a mut Layouts,
    stretch: &'a mut Stretch,
    parent: Option<layout::ParentLayout>,
}

impl Layouter<'_> {
    fn generate_tree(mut self) {
        let layout = self.widgets[self.id.widget].layout(Layouter {
            id: self.id,
            widgets: &self.widgets,
            graph: &self.graph,
            layouts: &mut self.layouts,
            stretch: &mut self.stretch,
            parent: self.parent,
        });

        println!("layout: {:#?}", self.id.widget);

        let (parent, layout_node) = match layout {
            Layout::Flex(style) => {
                let node = self.stretch.new_node(style, vec![]).unwrap();
                match self.parent {
                    Some(ParentLayout::Node { node: parent, root }) => {
                        println!("  relation: {:#?}", (parent, node));
                        self.stretch.add_child(parent, node).unwrap();
                        (
                            Some(ParentLayout::Node { node, root }),
                            LayoutNode::Flex { node, root },
                        )
                    }
                    _ => (
                        Some(ParentLayout::Node { node, root: node }),
                        LayoutNode::FlexRoot(node),
                    ),
                }
            }
            Layout::Pass => (self.parent, LayoutNode::None),
        };
        self.layouts[self.id.widget] = layout_node;

        // Layout the children
        for child in self.id.node.children(&self.graph) {
            let child_id = self.graph[child].data;
            println!("layout - child: {:#?}", (self.id.widget, child_id));
            Layouter {
                id: Id {
                    node: child,
                    widget: child_id,
                },
                widgets: &self.widgets,
                graph: &self.graph,
                layouts: &mut self.layouts,
                stretch: &mut self.stretch,
                parent,
            }
            .generate_tree();
        }
    }

    fn compute_layout(mut self, width: f32, height: f32) {
        let layout = &self.layouts[self.id.widget];

        let (width, height) = match layout {
            LayoutNode::FlexRoot(node) => {
                self.stretch
                    .compute_layout(
                        *node,
                        Size {
                            width: width.to_number(),
                            height: height.to_number(),
                        },
                    )
                    .unwrap();
                let layout = self.stretch.layout(*node).unwrap();
                println!("root: {:?}", layout);
                (layout.size.width, layout.size.height)
            }
            LayoutNode::Flex { node, .. } => {
                let layout = self.stretch.layout(*node).unwrap();
                println!("node: {:?} ({:?})", layout, node);
                (layout.size.width, layout.size.height)
            }
            LayoutNode::None => {
                println!("pass: {:?}", (width, height));
                (width, height)
            }
        };

        // Layout the children
        for child in self.id.node.children(&self.graph) {
            let child_id = self.graph[child].data;
            Layouter {
                id: Id {
                    node: child,
                    widget: child_id,
                },
                widgets: &self.widgets,
                graph: &self.graph,
                layouts: &mut self.layouts,
                stretch: &mut self.stretch,
                parent: None,
            }
            .compute_layout(width, height);
        }
    }
}

pub struct Build<'a> {
    pub id: Id,
    widgets: &'a Widgets,
    graph: &'a mut Graph,
    graph_map: &'a mut GraphMap,
    widget_ids: &'a mut WidgetIdGen,
    keys: &'a mut Keys,
    layouts: &'a mut Layouts,
    states: &'a mut StateCache,
}

type StateInternal = Rc<RefCell<Box<Any>>>; // TODO: high very meh

impl<'a> Build<'a> {
    fn clone(&mut self) -> Build {
        Build {
            id: self.id,
            widgets: self.widgets,
            graph: self.graph,
            graph_map: self.graph_map,
            widget_ids: self.widget_ids,
            keys: self.keys,
            layouts: self.layouts,
            states: self.states,
        }
    }

    fn new_widget(&mut self) -> Id {
        let widget = self.widget_ids.generate();
        let node = self.graph.new_node(widget);
        self.graph_map.insert(widget, node);
        Id { widget, node }
    }

    fn build(&mut self) {
        // println!("build :: build {:?}", self.id);
        self.widgets[self.id.widget].build(self.clone());
    }

    fn append_child(&mut self, parent: NodeId, child: NodeId) {
        println!("build :: append {:?} {:?}", parent, child);
        parent.append(child, self.graph).unwrap();
    }

    pub unsafe fn add<W: Widget>(&mut self, widget: W, global_key: Option<Key>) {
        let id = self.new_widget();

        if let Some(_) = global_key {
            let prev = *self
                .states
                .prev_interns
                .get(&id.widget)
                .unwrap_or(&id.widget);
            self.states.next_interns.insert(id.widget, prev);
        }

        assert_eq!(id.widget.0, self.widgets.push(Box::new(widget)));
        assert_eq!(id.widget.0, self.keys.push(global_key));
        assert_eq!(id.widget.0, self.layouts.push(LayoutNode::None));

        self.append_child(self.id.node, id.node);
        self.id = id;
        self.build();
    }

    pub unsafe fn add_view(&mut self, view: &View) {
        let id = self.new_widget();

        let ref_view = view::ViewInner::Ref(id);
        let owned = std::mem::replace(&mut *view.0.borrow_mut(), ref_view);
        match owned {
            view::ViewInner::Owned(widget) => {
                assert_eq!(id.widget.0, self.widgets.push(widget));
                assert_eq!(id.widget.0, self.layouts.push(LayoutNode::None));
            }
            view::ViewInner::Ref(_) => panic!(),
        }

        self.append_child(self.id.node, id.node);
        self.id = id;
        self.build();
    }

    pub fn create_state<S, F>(&mut self, default: F) -> StateInternal
    where
        F: FnOnce() -> S,
        S: 'static,
    {
        let state_ty = TypeId::of::<S>();
        let key = self.keys[self.id.widget].unwrap();
        let query_key = match key {
            Key::Global(_) => Some(key),
            Key::Local { parent, local } => {
                let orig_parent_id = self.states.next_interns.get(&parent);
                if let Some(orig_parent) = orig_parent_id {
                    Some(Key::Local {
                        parent: *orig_parent,
                        local,
                    })
                } else {
                    Some(key)
                }
            }
        };

        if let Some(key) = query_key {
            match self.states.states.entry(key) {
                Entry::Occupied(mut value) => {
                    if value.get().ty != state_ty {
                        *value.get_mut() = StateStorage {
                            ty: state_ty,
                            data: Rc::new(RefCell::new(Box::new(default()))),
                        };
                    }

                    value.get().data.clone()
                }
                Entry::Vacant(entry) => entry
                    .insert(StateStorage {
                        ty: state_ty,
                        data: Rc::new(RefCell::new(Box::new(default()))),
                    })
                    .data
                    .clone(),
            }
        } else {
            self.states.states.insert(
                key,
                StateStorage {
                    ty: state_ty,
                    data: Rc::new(RefCell::new(Box::new(default()))),
                },
            );
            self.states.states[&key].data.clone()
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id {
    pub widget: WidgetId,
    pub node: NodeId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(pub(crate) usize);

#[derive(Debug)]
struct WidgetIdGen {
    next: usize,
}
impl WidgetIdGen {
    pub fn new() -> Self {
        WidgetIdGen { next: 0 }
    }

    pub fn generate(&mut self) -> WidgetId {
        let id = self.next;
        self.next += 1;
        WidgetId(id)
    }
}

#[derive(Debug)]
struct StateStorage {
    ty: TypeId,
    data: StateInternal,
}

type InternMap = HashMap<WidgetId, WidgetId>;

// When a new state gets created the associated intern id of the widget will be stored.
//
// The intern id will change with each rebuild, so we need to track the mapping between
// the intern id in the current build pass and the original intern id used during state creation.
// `prev_intern` will contain the mapping information for the previous build pass.
// `next_intern` will contain the new connections in the current build pass.
// Afterwards `next_inter` will be replace the contents of `prev_intern`.
#[derive(Debug)]
struct StateCache {
    states: HashMap<Key, StateStorage>,
    prev_interns: InternMap,
    next_interns: InternMap,
}
impl StateCache {
    pub fn new() -> Self {
        StateCache {
            states: HashMap::new(),
            prev_interns: InternMap::new(),
            next_interns: InternMap::new(),
        }
    }
}
