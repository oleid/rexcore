use std::fmt::Debug;
use std::ops::FnMut;
use std::fmt;

pub struct EventSource<T: Send + Clone + Debug> {
    targets: Vec<Box<FnMut(T)>>,
}

impl<T: Send + Clone + Debug> EventSource<T> {
    pub fn new() -> EventSource<T> {
        EventSource::<T> {
            targets: Vec::new(),
        }
    }

    pub fn fire(&mut self, val: T) {
        debug!("Firing {:?} to {} sinks", val, self.targets.len());
        for tg in self.targets.iter_mut() {
            debug!("\tFiring: -- ");
            (tg)(val.clone());
        }
    }

    pub fn push(&mut self, callback: Box<FnMut(T)>) {
        self.targets.push(callback);
    }
}

impl<T: Send + Clone + Debug> Debug for EventSource<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<EventSource with {} targets>", self.targets.len())
    }
}

