#![feature(async_closure)]

use icmp_encoding::{
    self,
    IcmpV4Header,
    IpV4PacketHeader,
    WritableIcmpPacket,
};
use std::{
    net::Ipv4Addr,
    sync::Arc,
    time::{
        Duration,
        Instant,
    },
};
use tokio;
use tokio_icmp::IcmpV4Socket;

#[tokio::test]
async fn ping_pong() -> Result<(), Box<dyn std::error::Error>> {
    let socket = IcmpV4Socket::new()?;

    let mut raw_send_buffer = &mut [1u8; 64][..];
    let mut send_buffer = WritableIcmpPacket::new(&mut raw_send_buffer).unwrap();

    let ping_message = IcmpV4Header {
        message_type: 8,
        code: 0,
        remaining: 0,
    };

    ping_message.encode(&mut send_buffer);

    let dest_addr = Ipv4Addr::new(8, 8, 8, 8);
    socket.send_to(send_buffer.full(), dest_addr).await?;

    let mut recv_buffer = &mut [0u8; 1500][..];
    let (bytes_read, from) = socket.recv_from(&mut recv_buffer).await?;

    let (ip_header, body) = IpV4PacketHeader::decode(&recv_buffer[..bytes_read]).unwrap();
    let (icmp_header, data) = IcmpV4Header::decode(body).unwrap();

    assert_eq!(
        (ip_header, from, icmp_header, data),
        (
            IpV4PacketHeader { protocol: 1 },
            dest_addr,
            IcmpV4Header {
                message_type: 0,
                code: 0,
                remaining: 0
            },
            &[1; 64 - 8][..],
        )
    );

    Ok(())
}

#[tokio::test(threaded_scheduler)]
async fn ping_pong_many() {
    // let dest_addr = Ipv4Addr::new(10, 100, 1, 1);
    let dest_addr = Ipv4Addr::new(10, 100, 26, 1);
    // let dest_addr = Ipv4Addr::new(8, 8, 8, 8);

    // MTU or size of the largest packet we can send without fragmentation.
    const PACKET_SIZE: usize = 1500;

    let socket = Arc::new(IcmpV4Socket::new().unwrap());
    let recv_socket = socket.clone();

    let future = tokio::task::spawn((|| async move {
        let mut recv_buffer = &mut [0u8; PACKET_SIZE][..];
        let start = Instant::now();

        let mut count = 0u32;

        loop {
            let timeout_result = tokio::time::timeout(
                Duration::from_millis(1000),
                recv_socket.recv_from(&mut recv_buffer),
            )
            .await;

            let (bytes_read, from) = match timeout_result {
                Err(_) => break,
                Ok(inner) => inner.unwrap(),
            };

            let (ip_header, body) = IpV4PacketHeader::decode(&recv_buffer[..bytes_read]).unwrap();
            let (icmp_header, data) = IcmpV4Header::decode(body).unwrap();

            count = count + 1;
        }

        println!("{:?}", (count, Instant::now().duration_since(start)));
    })());

    let mut raw_send_buffer = &mut [1u8; PACKET_SIZE - 20][..];
    let mut send_buffer = WritableIcmpPacket::new(&mut raw_send_buffer).unwrap();

    let ping_message = IcmpV4Header {
        message_type: 8,
        code: 0,
        remaining: 0,
    };

    ping_message.encode(&mut send_buffer);

    for _ in 0u64..100000 {
        socket.send_to(send_buffer.full(), dest_addr).await.unwrap();
    }

    future.await.unwrap();

    // TODO: Find MTU
    // TODO: Find Gateway

    // TODO: Actually Seem to be Saturing Link With Ping?

    // TODO: The network is so bad that packets get dropped before the buffer is
    // anywhere near full

    // TODO: Getting very accurate speed tests
    // TODO: How does android monitor network link state?

    // It seems like ios gg
    // https://lists.apple.com/archives/macnetworkprog/2006/Jul/msg00017.html

    // TODO: Discovering the MTU in a cross platform way is hard.

    // TODO: Send the biggest packets because

    // TODO: Somehow TCP is more efficient at saturating the link.
    // TODO: TCP backs off and doesn't send on a saturated connection
    // TODO: this is why tcp is the best way to saturate a link

    // TODO: potentially terminal bandwidth?
    // not the case: 30Mbps vs 50Mbps

    // TODO: sending as fast as possible means congestion collapse will happen
    // TODO: UDP trades of bandwidth for speed

    // TODO: Saturating from the sending end is possible 300Mb can be sent per
    // second, but the incoming traffic is much lesser and then the driver
    // crashed
}
