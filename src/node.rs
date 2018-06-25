use std::cell::{RefCell, RefMut};
use std::rc::Rc;

/// A node object, which has interior mutability and holds a shared reference to some object
///
/// Used to hide the details of the sink! macro.
#[derive(Debug)]
pub struct Node<Obj> {
    obj: Rc<RefCell<Obj>>,
}

impl<Obj> Node<Obj> {
    pub fn new(obj: Obj) -> Node<Obj> {
        Node {
            obj: Rc::new(RefCell::new(obj)),
        }
    }

    /// Borrow the wrapped object mutably. Used for calling a function via sink!
    pub fn get_obj(&self) -> RefMut<Obj> {
        self.obj.borrow_mut()
    }

    /// Clone the underlying object. Used for putting a reference into the EventSource.
    pub fn clone_obj(&self) -> Rc<RefCell<Obj>> {
        self.obj.clone()
    }
}

impl<Obj> Clone for Node<Obj> {
    // Manual implementation; Obj needs no Clone, since it's in a Rc
    fn clone(&self) -> Self {
        Node {
            obj: self.clone_obj(),
        }
    }
}

/// Takes a node containing some object and a function name
/// and creates a closure calling the function of that object.
/// To make this work, the closure takes ownership of a clone
/// of the refcounted object in the node. The object, in turn,
/// is contained in a ref-cell, which is needed for run-time
/// borrow-checking.
///
/// This won't compile if source and sink(i.e. the node)
/// live in different threads. In this case, probably something
/// like src =>> region_barrier_sink | mpc_channel | region_barrier_source =>> actual_sink
/// would be needed.
/// ```
/// # #[macro_use] extern crate rexcore;
/// # use rexcore::*;
///
/// struct Mock { activated : bool }
///
/// impl Mock {
///     fn activate(&mut self, really : bool) { self.activated = really; }
/// }
/// # fn main() {
///
/// let mut e = EventSource::new();
/// let mut m = Node::new( Mock { activated : false } );
///
/// connect!(e =>> sink!( at m, call Mock::activate) );
///
/// assert_eq!(false, m.get_obj().activated);
///
/// e.fire(true);
///
/// assert_eq!(true, m.get_obj().activated);
/// # }
/// ```
#[macro_export]
macro_rules! sink {
    (at $node:ident,call $func:expr) => {{
        let obj = $node.clone_obj();
        move |v| ($func)(&mut obj.borrow_mut(), v)
    }};
}
