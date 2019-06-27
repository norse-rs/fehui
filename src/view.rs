use crate::{Id, Widget};
use std::cell::RefCell;

#[derive(Debug)]
pub struct View(pub(crate) RefCell<ViewInner>);

pub enum ViewInner {
    Owned(Box<Widget>),
    Ref(Id),
}

impl std::fmt::Debug for ViewInner {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ViewInner::Owned(_) => write!(f, "View::Owned"),
            ViewInner::Ref(ref id) => write!(f, "View::Ref({:?})", id),
        }
    }
}

impl<W: Widget> From<W> for View {
    fn from(widget: W) -> Self {
        View(RefCell::new(ViewInner::Owned(Box::new(widget))))
    }
}

#[macro_export]
macro_rules! views {
    ($elem:expr; $n:expr) => (
        $crate::vec::from_elem($elem.into(), $n)
    );
    ($($x:expr),*) => (
        <[_]>::into_vec(Box::new([$($x.into()),*]))
    );
    ($($x:expr,)*) => (vec![$($x.into()),*])
}
