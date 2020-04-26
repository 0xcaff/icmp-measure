use icmp_encoding::{
    self,
    IcmpV4Header,
    IpV4PacketHeader,
    WritableIcmpPacket,
};
use std::net::Ipv4Addr;
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

    // TODO: Better High Level Abstraction for Sending Ping
}
