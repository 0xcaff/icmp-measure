use crate::evented_socket;
use futures::{
    future::poll_fn,
    ready,
};
use mio::Ready;
use socket2::{
    Domain,
    Protocol,
    SockAddr,
    Type,
};
use std::{
    io,
    net::{
        Ipv4Addr,
        SocketAddr,
        SocketAddrV4,
    },
    task::{
        Context,
        Poll,
    },
};
use tokio::io::PollEvented;

/// Exposes a futures compatible API for sending and receiving ICMPv4 ping
/// messages.
///
/// Polling multiple send futures or multiple receive futures at the same time
/// is undefined behavior.
///
/// From `PollEvented` docs:
///
/// While `PollEvented` is `Sync` (if the underlying I/O type is `Sync`), the
/// caller must ensure that there are at most two tasks that use a `PollEvented`
/// instance concurrently. One for reading and one for writing. While violating
/// this requirement is "safe" from a Rust memory model point of view, it will
/// result in unexpected behavior in the form of lost notifications and tasks
/// hanging.
pub struct IcmpV4Socket {
    io: PollEvented<evented_socket::Socket>,
}

impl IcmpV4Socket {
    pub fn new() -> io::Result<Self> {
        IcmpV4Socket::from_metal_socket(evented_socket::Socket::new(
            Domain::ipv4(),
            Type::dgram(),
            Protocol::icmpv4(),
        )?)
    }

    fn from_metal_socket(socket: evented_socket::Socket) -> io::Result<Self> {
        let io = PollEvented::new(socket)?;
        Ok(IcmpV4Socket { io })
    }

    pub async fn send_to(&self, buf: &[u8], target: Ipv4Addr) -> io::Result<usize> {
        poll_fn(|cx| self.poll_send_to(cx, buf, &SockAddr::from(SocketAddrV4::new(target, 0))))
            .await
    }

    fn poll_send_to(
        &self,
        cx: &mut Context,
        buf: &[u8],
        target: &SockAddr,
    ) -> Poll<io::Result<usize>> {
        ready!(self.io.poll_write_ready(cx))?;

        match self.io.get_ref().send_to(buf, target) {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.io.clear_write_ready(cx)?;
                Poll::Pending
            }
            x => Poll::Ready(x),
        }
    }

    pub async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, Ipv4Addr)> {
        let (bytes_written, from_addr) = poll_fn(|cx| self.poll_recv_from(cx, buf)).await?;

        Ok((
            bytes_written,
            *from_addr.as_std().unwrap().into_v4().unwrap().ip(),
        ))
    }

    fn poll_recv_from(
        &self,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SockAddr)>> {
        ready!(self.io.poll_read_ready(cx, Ready::readable()))?;

        match self.io.get_ref().recv_from(buf) {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.io.clear_read_ready(cx, Ready::readable())?;
                Poll::Pending
            }
            x => Poll::Ready(x),
        }
    }
}

trait AsV4Address {
    fn into_v4(self) -> Option<SocketAddrV4>;
}

impl AsV4Address for SocketAddr {
    fn into_v4(self) -> Option<SocketAddrV4> {
        match self {
            SocketAddr::V4(addr) => Some(addr),
            SocketAddr::V6(_addr) => None,
        }
    }
}
