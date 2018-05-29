use std::cell::{RefCell, RefMut};
use std::fmt::Debug;
use std::ops::FnMut;
use std::rc::Rc;

struct EventSource<T: Send + Clone + Debug> {
    targets: Vec<Box<FnMut(T)>>,
}

impl<T: Send + Clone + Debug> EventSource<T> {
    pub fn new() -> EventSource<T> {
        EventSource::<T> {
            targets: Vec::new(),
        }
    }

    fn fire(&mut self, val: T) {
        for tg in self.targets.iter_mut() {
            print!("Firing: -- ");
            (tg)(val.clone());
        }
    }
}

// Takes a node containing an object and a function name
// and creates a closure calling the function of that object.
// To make this work, the closure takes ownership of a clone
// of the refcounted object in the node. The object, in turn,
// is contained in a ref-cell, which is needed for run-time 
// borrow-checking.
//
// This won't compile if source and sink(i.e. the node)
// live in different threads. In this case, probably something
// like src =>> region_barrier_sink | mpc_channel | region_barrier_source =>> actual_sink
// would be needed. 
macro_rules! sink {
    (at $node:ident,call $func:expr) => {{
        let obj = $node.clone_obj();
        move |v| ($func)(&mut obj.borrow_mut(), v)
    }};
}

// Recursively call callables implementing some Fn trait
macro_rules! chain {
    // -- call $dest with the result of $src
    ( $src:expr =>> $dest:expr) => {
        {
            // we need to assign the closures to values, before
            // creating the new closure, in order to make sink!
            // work. Otherwise the entire $node is moved into the lambda.
            let f1 = $src;
            let f2 = $dest;

            move |v| f2(f1(v))
        }
    };
    // --- Handle the rest recursively
    ($src:expr =>> $dest:expr  $(=>> $dests:expr)+ ) => {
        chain! {
            chain! { $src =>> $dest }
            $(=>> $dests )+
        }
    };
}

macro_rules! connect {
    // Use the above's chain macro to handle multiple parameters
    ($src:expr  =>> $trans1:expr $(=>> $transes:expr)+ ) => {{
        connect!( $src =>>  chain!($trans1 $(=>> $transes)+ ) )
    }};
    ( $src:expr =>> $dest:expr ) => {
        $src.targets.push( Box::new($dest));
    };
}

struct Akkumulator {
    accum: usize,
    out_accum: EventSource<usize>,
}

impl Akkumulator {
    pub fn new() -> Akkumulator {
        Akkumulator {
            accum: 0,
            out_accum: EventSource::new(),
        }
    }

    fn accumulate(&mut self, val: usize) {
        println!(
            "Akkumulator: Hab {} bekommen, hab bereits {}!)",
            val, self.accum
        );
        self.accum += val;
        self.out_accum.fire(self.accum);
    }

    fn out_accumulator(&mut self) -> &mut EventSource<usize> {
        &mut self.out_accum
    }
}

struct Node<Obj> {
    _name: String,
    obj: Rc<RefCell<Obj>>,
}

impl<Obj> Node<Obj> {
    pub fn new(_name: String, obj: Obj) -> Node<Obj> {
        Node {
            _name,
            obj: Rc::new(RefCell::new(obj)),
        }
    }

    pub fn get_obj(&self) -> RefMut<Obj> {
        self.obj.borrow_mut()
    }

    pub fn clone_obj(&self) -> Rc<RefCell<Obj>> {
        self.obj.clone()
    }
}

fn main() {
    let closure = |v: usize| v * v + 1;
    let complicated_calculation = chain!( |a : usize| 2*a
        =>> |b| 3*b
        =>>|c| 4*c
        =>> closure);

    let mut e = EventSource::new();

    connect!(e
        =>> complicated_calculation
        =>> |s| println!("Ergebnis der komplizierten Rechnung: {}", s)
    );
    e.fire(12345);

    // Akku-Beispiel
    let akku = Node::new("Akkumulator".to_owned(), Akkumulator::new());

    connect!(e
        =>> |v| 3.141 * v as f32
        =>> |w : f32| w.sqrt()
        =>> |x| println!("{}", x )
    );

    connect!(e
        =>> | v | { println!("Just inspecting {}", v); v }
        =>> sink!(at akku, call Akkumulator::accumulate)
    );

    connect!(
        akku.get_obj().out_accumulator()
        =>> |a| 2*a
        =>> |b| b / 2
        =>> |v| println!("Akku ist geladen zu {}", v) 
    );

    for i in 1..6 {
        e.fire(i);
    }
}
