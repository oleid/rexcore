#[macro_use]
extern crate rexcore;

use rexcore::*;

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
    let akku = Node::new(Akkumulator::new());

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
