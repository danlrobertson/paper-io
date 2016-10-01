use libc::{self, c_int, epoll_event, epoll_create1, epoll_ctl, epoll_wait};
use libc::{EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD};
use paper::{Event, Events};
use std::io::Error;

impl Event {
    pub fn raw(self) -> u32 {
        match self {
            Event::Read => libc::EPOLLIN as u32,
            Event::Write => libc::EPOLLOUT as u32
        }
    }
}

pub struct Selector {
    epollfd: c_int,
    allfds: Vec<c_int>
}

impl Selector {
    pub fn new() -> Selector {
        Selector {
            epollfd: unsafe { epoll_create1(0) },
            allfds: Vec::new()
        }
    }

    pub fn add(&mut self, fd: c_int, event: Events) -> c_int {
        self.allfds.push(fd);
        let mut ev = epoll_event {
            events: event.events,
            u64: fd as u64
        };
        unsafe {
            epoll_ctl(self.epollfd, EPOLL_CTL_ADD, fd, &mut ev as *mut epoll_event);
        }
        fd
    }

    pub fn modify(&mut self, fd: c_int, event: Events) -> c_int {
        assert!(self.allfds.contains(&fd));
        let mut ev = epoll_event {
            events: event.events,
            u64: fd as u64
        };
        unsafe {
            epoll_ctl(self.epollfd, EPOLL_CTL_MOD, fd, &mut ev as *mut epoll_event);
        }
        fd
    }

    pub fn rm(&mut self, fd: c_int) -> c_int {
        let res = unsafe {
            // NB: See BUGS portion of the epoll_ctl man page
            // for the purpose of the ev variable
            let mut ev = epoll_event {
                events: 0,
                u64: 0
            };
            epoll_ctl(self.epollfd, EPOLL_CTL_DEL, fd, &mut ev as *mut epoll_event)
        };
        self.allfds.retain(|local_fd| *local_fd != fd);
        res
    }

    pub fn select(&self, cap: usize, timeout: Option<i32>) -> Result<Box<Iterator<Item=c_int>>, Error> {
        let events = unsafe {
            let mut events = Vec::with_capacity(cap);
            let timeout = match timeout {
                Some(time) => time,
                None => -1
            };
            let nfds = epoll_wait(self.epollfd, events.as_mut_ptr(), cap as i32, timeout);
            if nfds < 0 {
                return Err(Error::last_os_error());
            }
            events.set_len(nfds as usize);
            events
        };
        Ok(Box::new(events.into_iter().map(|evt| { evt.u64 as c_int })))
    }
}

impl Drop for Selector {
    fn drop(&mut self) {
        unsafe {
            for fd in self.allfds.iter() {
                libc::close(*fd);
            }
            libc::close(self.epollfd);
        }
    }
}
