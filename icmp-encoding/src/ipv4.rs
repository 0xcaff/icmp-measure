use nom::{
    bits::{
        bits,
        bytes,
        complete::{
            tag,
            take,
        },
    },
    error::ErrorKind,
    number::complete::be_u16,
    Err,
    IResult,
};

#[derive(PartialEq, Debug)]
pub struct IpV4PacketHeader {
    pub protocol: u16,
}

const IP_VERSION: u8 = 4;

impl IpV4PacketHeader {
    pub fn decode(packet: &[u8]) -> Result<(IpV4PacketHeader, &[u8]), Err<(&[u8], ErrorKind)>> {
        let (remaining, header) = bits(|state| -> IResult<_, IpV4PacketHeader, (_, ErrorKind)> {
            let (state, _) = tag(IP_VERSION, 4usize)(state)?;
            let (state, internet_header_length) = take::<_, u8, _, _>(4usize)(state)?;
            let (state, _) = take::<_, u8, _, _>(8usize)(state)?;
            let (state, _total_length) = bytes::<_, _, (&[u8], ErrorKind), _, _>(be_u16)(state)?;

            let (state, _) = take::<_, u32, _, _>(32usize)(state)?;

            let (state, _time_to_live) = take::<_, u8, _, _>(8usize)(state)?;
            let (state, protocol) = take(8usize)(state)?;
            let (state, _header_checksum) = take::<_, u16, _, _>(16usize)(state)?;

            let (state, _source_ip_address) = take::<_, u32, _, _>(32usize)(state)?;

            let (state, _dest_ip_address) = take::<_, u32, _, _>(32usize)(state)?;

            let (state, _options) =
                take::<_, usize, _, _>((internet_header_length - 5) * 32)(state)?;

            Ok((state, IpV4PacketHeader { protocol }))
        })(packet)?;

        Ok((header, remaining))
    }
}

#[cfg(test)]
mod test {
    use super::IpV4PacketHeader;

    fn encoded() -> [u8; 84] {
        [
            69, 0, 64, 0, 101, 92, 0, 0, 64, 1, 253, 194, 10, 100, 1, 1, 10, 100, 1, 194, 0, 0,
            131, 227, 32, 5, 0, 15, 94, 165, 50, 160, 0, 0, 223, 191, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
            38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
        ]
    }

    #[test]
    fn decode() {
        let encoded = &encoded()[..];
        let decoded = IpV4PacketHeader::decode(encoded).unwrap();

        assert_eq!((IpV4PacketHeader { protocol: 1 }, &encoded[20..]), decoded)
    }
}
