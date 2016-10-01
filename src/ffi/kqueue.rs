use libc::{self, c_int, kqueue, timespec};
use libc::{EV_ADD, NOTE_WRITE, EV_DELETE};
use std::io::Error;
use std::mem;
use std::ptr;
use paper::{Event, Events};

impl Event {
    pub fn raw(self) -> u32 {
        match self {
            Event::Read => libc::EVFILT_READ as u32,
            Event::Write => libc::EVFILT_WRITE as u32
        }
    }
}

pub struct Selector {
    kqueuefd: c_int,
    allfds: Vec<c_int>
}

impl Selector {
    pub fn new() -> Selector {
        Selector {
            kqueuefd: unsafe { kqueue() },
            allfds: Vec::new()
        }
    }

    pub fn add(&mut self, fd: c_int, event: Events) -> c_int {
        self.allfds.push(fd);
        unsafe {
            let mut ev: libc::kevent = mem::zeroed();
            ev.ident = fd as usize;
            ev.filter = event.events as i16;
            ev.flags = EV_ADD;
            ev.fflags = NOTE_WRITE;
            libc::kevent(self.kqueuefd, &ev, 1, ptr::null_mut(), 0, ptr::null());
        }
        fd
    }

    pub fn modify(&mut self, fd: c_int, event: Events) -> c_int {
        assert!(self.allfds.contains(&fd));
        unsafe {
            let mut ev: libc::kevent = mem::zeroed();
            ev.ident = fd as usize;
            ev.filter = event.events as i16;
            ev.flags = EV_ADD;
            ev.fflags = NOTE_WRITE;
            libc::kevent(self.kqueuefd, &ev, 1, ptr::null_mut(), 0, ptr::null());
        }
        fd
    }

    pub fn rm(&mut self, fd: c_int) -> c_int {
        let res = unsafe {
            let mut ev: libc::kevent = mem::zeroed();
            ev.ident = fd as usize;
            ev.flags = EV_DELETE;
            ev.fflags = NOTE_WRITE;
            libc::kevent(self.kqueuefd, &ev, 1, ptr::null_mut(), 0, ptr::null())
        };
        self.allfds.retain(|local_fd| *local_fd != fd);
        res
    }

    pub fn select(&self, cap: usize, timeout: Option<i32>) -> Result<Box<Iterator<Item=c_int>>, Error> {
        let events = unsafe {
            let mut events: Vec<libc::kevent> = Vec::with_capacity(cap);
            let mut time = timespec {
                tv_sec: 0,
                tv_nsec: 0
            };
            let timeout: *const libc::timespec = match timeout {
                Some(secs) => {
                    time.tv_sec = secs as i64;
                    &time
                },
                None => ptr::null(),
            };
            let nfds = libc::kevent(self.kqueuefd, ptr::null_mut(), 0,
                                    events.as_mut_ptr() as *mut libc::kevent,
                                    cap as i32, timeout);
            if nfds < 0 {
                return Err(Error::last_os_error());
            }
            events.set_len(nfds as usize);
            events
        };
        Ok(Box::new(events.into_iter().map(|evt| { evt.ident as c_int })))
    }
}

impl Drop for Selector {
    fn drop(&mut self) {
        unsafe {
            for fd in self.allfds.iter() {
                libc::close(*fd);
            }
            libc::close(self.kqueuefd);
        }
    }
}
