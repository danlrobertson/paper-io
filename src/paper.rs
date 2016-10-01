use ffi;
use libc::{c_int};
use std::io::Error;

pub enum Event {
    Read,
    Write
}

pub struct Events {
    pub events: u32
}

impl Events {
    pub fn new(event: Event) -> Events {
        Events {
            events: event.raw()
        }
    }

    pub fn add(&mut self, event: Event) -> &mut Events {
        self.events = self.events | event.raw();
        self
    }

    pub fn rm(&mut self, event: Event) -> &mut Events {
        self.events = self.events & !event.raw();
        self
    }

    pub fn raw(&self) -> u32 {
        self.events
    }
}

pub struct Selector {
    selector: ffi::Selector,
    cap: usize,
    wait: Option<i32>
}

impl Selector {
    pub fn new(cap: usize, wait: Option<i32>) -> Selector {
        Selector {
            selector: ffi::Selector::new(),
            cap: cap,
            wait: wait
        }
    }

    pub fn add(&mut self, fd: c_int, event: Events) -> c_int {
        self.selector.add(fd, event)
    }

    pub fn modify(&mut self, fd: c_int, event: Events) -> c_int {
        self.selector.modify(fd, event)
    }

    pub fn rm(&mut self, fd: c_int) -> c_int {
        self.selector.rm(fd)
    }
    pub fn select(&self) -> Result<Box<Iterator<Item=c_int>>, Error> {
        self.selector.select(self.cap, self.wait)
    }
}
