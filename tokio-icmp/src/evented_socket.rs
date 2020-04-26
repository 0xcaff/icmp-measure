use mio::{
    unix::EventedFd,
    Evented,
    Poll,
    PollOpt,
    Ready,
    Token,
};
use socket2::{
    Domain,
    Protocol,
    SockAddr,
    Socket as SysSocket,
    Type,
};
use std::{
    io,
    os::unix::io::{
        AsRawFd,
        RawFd,
    },
};

/// A low-level non-blocking socket.
pub struct Socket {
    sys: SysSocket,
}

impl Socket {
    pub fn new(domain: Domain, type_: Type, protocol: Protocol) -> io::Result<Self> {
        let socket = SysSocket::new(domain, type_, Some(protocol))?;
        socket.set_nonblocking(true)?;

        Ok(Self { sys: socket })
    }

    pub fn send_to(&self, buf: &[u8], target: &SockAddr) -> io::Result<usize> {
        self.sys.send_to(buf, target)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SockAddr)> {
        self.sys.recv_from(buf)
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.sys.as_raw_fd()
    }
}

impl Evented for Socket {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}
