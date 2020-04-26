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

#[derive(Debug)]
pub struct IpV4PacketHeader {
    protocol: u16,
}

const IP_VERSION: u8 = 4;

impl IpV4PacketHeader {
    pub fn decode(packet: &[u8]) -> Result<(IpV4PacketHeader, &[u8]), Err<(&[u8], ErrorKind)>> {
        let (remaining, header) = bits(|state| -> IResult<_, IpV4PacketHeader, (_, ErrorKind)> {
            let (state, _) = tag(IP_VERSION, 4usize)(state)?;
            let (state, internet_header_length) = take::<_, u8, _, _>(4usize)(state)?;
            let (state, _) = take::<_, u8, _, _>(8usize)(state)?;
            let (state, total_length) = bytes::<_, _, (&[u8], ErrorKind), _, _>(be_u16)(state)?;

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
