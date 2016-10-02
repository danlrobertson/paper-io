use libc::{self, c_int, poll, pollfd, nfds_t, c_short};
use paper::{Event, Events};
use std::io::Error;

impl Event {
    pub fn raw(self) -> u32 {
        match self {
            Event::Read => libc::POLLIN as u32,
            Event::Write => libc::POLLOUT as u32
        }
    }
}

pub struct Selector {
    allfds: Vec<pollfd>,
    events: c_short,
}

impl Selector {
    pub fn new() -> Selector {
        Selector {
            allfds: Vec::new(),
            events: 0
        }
    }

    pub fn add(&mut self, fd: c_int, event: Events) -> c_int {
        self.events |= event.events as c_short;
        self.allfds.push(pollfd {
            fd: fd,
            events: event.events as c_short,
            revents: 0,
        });
        fd
    }

    pub fn modify(&mut self, fd: c_int, event: Events) -> c_int {
        self.events |= event.events as c_short;
        for pollfd in self.allfds.iter_mut() {
            if fd == pollfd.fd {
                pollfd.events = event.events as c_short;
                return pollfd.fd;
            }
        }
        -1
    }

    pub fn rm(&mut self, fd: c_int) -> c_int {
        self.allfds.retain(|local_fd| local_fd.fd != fd);
        fd
    }

    pub fn select(&self, _: usize, timeout: Option<i32>) -> Result<Box<Iterator<Item=c_int>>, Error> {
        let timeout = match timeout {
            Some(time) => time,
            None => -1
        };
        let mut tmp_vec = self.allfds.clone();
        let res = unsafe {
            poll(tmp_vec.as_mut_ptr(), tmp_vec.len() as nfds_t, timeout)
        };
        if res <= 0 {
            return Err(Error::last_os_error());
        }
        tmp_vec.retain(|pollfd| (pollfd.revents & self.events) >= 0);
        for pollfd in &mut tmp_vec {
            pollfd.revents = pollfd.revents & !libc::POLLIN;
        }
        Ok(Box::new(tmp_vec.into_iter().map(|pollfd| pollfd.fd)))
    }
}

impl Drop for Selector {
    fn drop(&mut self) {
        unsafe {
            for pollfd in self.allfds.iter() {
                libc::close(pollfd.fd);
            }
        }
    }
}
