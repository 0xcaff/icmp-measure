mod icmp;
mod ipv4;

pub use icmp::{
    IcmpV4Header,
    WritableIcmpPacket,
};
pub use ipv4::IpV4PacketHeader;

#[cfg(test)]
mod test {
    use crate::{
        IcmpV4Header,
        IpV4PacketHeader,
    };

    #[test]
    fn decode_ping_request_packet() -> Result<(), Box<dyn ::std::error::Error>> {
        let encoded: [u8; 84] = [
            69, 0, 64, 0, 101, 92, 0, 0, 64, 1, 253, 194, 10, 100, 1, 1, 10, 100, 1, 194, 0, 0,
            131, 227, 32, 5, 0, 15, 94, 165, 50, 160, 0, 0, 223, 191, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
            38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
        ];

        let (ip_header, ip_data) = IpV4PacketHeader::decode(&encoded).unwrap();
        println!("{:?}, {:?}", ip_header, ip_data);

        let (icmp_header, icmp_data) = IcmpV4Header::decode(ip_data).unwrap();
        println!("{:?}, {:?}", icmp_header, icmp_data);

        Ok(())
    }
}
