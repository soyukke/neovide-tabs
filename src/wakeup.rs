#[cfg(unix)]
mod platform {
    use std::{
        io,
        os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
    };

    pub struct WakeupReceiver {
        fd: OwnedFd,
    }

    pub struct WakeupSender {
        fd: OwnedFd,
    }

    pub fn pipe() -> io::Result<(WakeupReceiver, WakeupSender)> {
        let mut descriptors = [0; 2];
        // SAFETY: `descriptors` points to two writable file-descriptor slots.
        if unsafe { libc::pipe(descriptors.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        for descriptor in descriptors {
            // SAFETY: fcntl operates on descriptors just created by pipe.
            unsafe {
                libc::fcntl(descriptor, libc::F_SETFD, libc::FD_CLOEXEC);
                let flags = libc::fcntl(descriptor, libc::F_GETFL);
                libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK);
            }
        }
        // SAFETY: both descriptors are newly owned results from `pipe`.
        let receiver = unsafe { OwnedFd::from_raw_fd(descriptors[0]) };
        // SAFETY: both descriptors are newly owned results from `pipe`.
        let sender = unsafe { OwnedFd::from_raw_fd(descriptors[1]) };
        Ok((WakeupReceiver { fd: receiver }, WakeupSender { fd: sender }))
    }

    impl WakeupReceiver {
        pub fn fd(&self) -> RawFd {
            self.fd.as_raw_fd()
        }

        pub fn clear(&self) {
            let mut buffer = [0_u8; 64];
            loop {
                // SAFETY: the buffer is writable and the descriptor is owned by `self`.
                let count = unsafe {
                    libc::read(
                        self.fd.as_raw_fd(),
                        buffer.as_mut_ptr().cast(),
                        buffer.len(),
                    )
                };
                if count <= 0 {
                    break;
                }
            }
        }
    }

    impl WakeupSender {
        pub fn notify(&self) {
            let byte = [1_u8];
            // SAFETY: the byte is readable and the descriptor is owned by `self`.
            unsafe {
                libc::write(self.fd.as_raw_fd(), byte.as_ptr().cast(), byte.len());
            }
        }
    }
}

#[cfg(not(unix))]
mod platform {
    use std::io;

    pub struct WakeupReceiver;
    pub struct WakeupSender;

    pub fn pipe() -> io::Result<(WakeupReceiver, WakeupSender)> {
        Ok((WakeupReceiver, WakeupSender))
    }

    impl WakeupReceiver {
        pub fn fd(&self) -> i32 {
            -1
        }

        pub fn clear(&self) {}
    }

    impl WakeupSender {
        pub fn notify(&self) {}
    }
}

pub use platform::{WakeupReceiver, WakeupSender, pipe};

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn wakeup_pipe_becomes_readable_and_can_be_cleared() {
        let (receiver, sender) = pipe().unwrap();
        sender.notify();
        assert!(is_readable(receiver.fd()));
        receiver.clear();
        assert!(!is_readable(receiver.fd()));
    }

    fn is_readable(fd: i32) -> bool {
        let mut descriptor = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: `descriptor` is a valid single-element pollfd array.
        (unsafe { libc::poll(&mut descriptor, 1, 0) }) == 1
            && descriptor.revents & libc::POLLIN != 0
    }
}
