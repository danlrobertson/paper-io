extern crate libc;
extern crate paper_io;

use libc::{socketpair, c_void, SOCK_SEQPACKET, AF_UNIX};
use paper_io::{Selector, Events, Event};

#[test]
fn select_simple() {
    let mut selector = Selector::new(10, None);
    let tx_data: [u8; 5] = [0x48, 0x65, 0x6c, 0x6c, 0x6f];
    let mut rx_data: [u8; 5] = [0, 0, 0, 0, 0];
    let mut fds = [0, 0];
    unsafe {
        let res = socketpair(AF_UNIX, SOCK_SEQPACKET, 0, &mut fds[0]);
        assert!(res >= 0);
        selector.add(fds[1], Events::new(Event::Read));
        let data = tx_data[..].as_ptr() as *const c_void;
        let res = libc::send(fds[0],
                             data,
                             tx_data.len(),
                             0);
        assert_eq!(res, 5);
    }
    for fd in selector.select().unwrap() {
        unsafe {
            assert_eq!(fd, fds[1]);
            let data = rx_data[..].as_mut_ptr() as *mut c_void;
            let data_len = libc::recv(fd,
                                      data,
                                      5,
                                      0);
            assert_eq!(data_len, 5);
        }
        assert_eq!(rx_data, tx_data);
    }
}

#[test]
fn select_simple_sentance() {
    let mut selector = Selector::new(10, None);
    let tx_data: [u8; 23] = [0x49, 0x74, 0x27, 0x73, 0x20, 0x6a, 0x75, 0x73,
                            0x74, 0x20, 0x61, 0x20, 0x66, 0x6c, 0x65, 0x73,
                            0x68, 0x20, 0x77, 0x6f, 0x75, 0x6e, 0x64];
    let mut rx_data: [u8; 23] = [0; 23];
    let mut fds = [0, 0];
    unsafe {
        let res = socketpair(AF_UNIX, SOCK_SEQPACKET, 0, &mut fds[0]);
        assert!(res >= 0);
        selector.add(fds[1], Events::new(Event::Read));
        let data = tx_data[..].as_ptr() as *const c_void;
        let res = libc::send(fds[0],
                             data,
                             tx_data.len(),
                             0);
        assert_eq!(res, 23);
    }
    for fd in selector.select().unwrap() {
        unsafe {
            assert_eq!(fd, fds[1]);
            let data = rx_data[..].as_mut_ptr() as *mut c_void;
            let data_len = libc::recv(fd,
                                      data,
                                      23,
                                      0);
            assert_eq!(data_len, 23);
        }
        assert_eq!(rx_data, tx_data);
    }
}
