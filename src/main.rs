#![feature(unboxed_closures, fn_traits)]

use std::fmt::Debug;
use std::ops::{FnMut, FnOnce};


// Currently unused, but probably this is needed to decouple
// multiple threads using a channel or something alike.

struct EventSink<T: Send + Clone + Debug> {
    callable: Box<FnMut(T)>,
}

// Implement traits FnOnce and FnMut, so that an event
// sink can be called like a function.

impl<T> FnOnce<(T,)> for EventSink<T>
where
    T: Send + Clone + Debug,
{
    type Output = ();
    extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output {
        self.sink(args.0)
    }
}

impl<T> FnMut<(T,)> for EventSink<T>
where
    T: Send + Clone + Debug,
{
    extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
        self.sink(args.0)
    }
}

impl<T: Send + Clone + Debug> EventSink<T> {
    pub fn new<F>(func: F) -> EventSink<T>
    where
        F: FnMut(T) + 'static, // still not sure about 'static, https://doc.rust-lang.org/error-index.html#E0309
    {
        EventSink::<T> {
            callable: Box::new(func),
        }
    }

    fn sink(&mut self, val: T) {
        println!("Sinking {:?}", val);
        (self.callable)(val);
    }
}

struct EventSource<T: Send + Clone + Debug> {
    targets: Vec<Box<FnMut(T)>>,
}

impl<T: Send + Clone + Debug> EventSource<T> {
    pub fn new() -> EventSource<T> {
        EventSource::<T> { targets: vec![] }
    }

    fn fire(&mut self, val: T) {
        for tg in self.targets.iter_mut() {
            print!("Firing: -- ");
            (tg)(val.clone());
        }
    }
}


// Recursively call callables implementing some Fn trait
macro_rules! chain {
    // -- Intermediate connection -- first function takes a value
    ( $src:expr =>> $dest:expr) => {
        move |v| ($dest)(  ($src)(v)  )
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
    ($src:expr =>> $dest:expr $(=>> $dests:expr)+ ) => {{
        connect!( $src =>>  chain!($dest $(=>> $dests)+ ) )
    }};
    ( $src:expr =>> $dest:expr ) => {
        $src.targets.push( Box::new($dest) );
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

fn main() {
    let closure = |v| v * v + 1;
    let complicated_calculation = chain!(
        |a| 2*a =>>
        |b| 3*b =>>
        |c| 4*c =>> closure);

    let mut e = EventSource::new();

    connect!(e
        =>> complicated_calculation
        =>> |s| println!("Ergebnis der komplizierten Rechnung: {}", s)
    );
    e.fire(12345);

    // Akku-Beispiel

    let mut akku = Akkumulator::new();

    // Hier ist der Knackpunkt: man kann die Reihenfolge nicht umdrehen,
    // da sonst akku. nimmer zugreifbar ist, da er in die event-source
    // rein-gemovt wurde. Und man braucht "move", weil sonst der Compiler
    // sagt, dass akku wohl nicht lange genug lebt.
    connect!(akku.out_accumulator()
        =>> |v| println!("\tAccum hat {} gesendet", v ));

    connect!(e =>> move |v| akku.accumulate(v));

    for i in 1..6 {
        e.fire(i);
    }
}
