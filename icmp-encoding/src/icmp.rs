use nom::{
    error::ErrorKind,
    number::complete::{
        be_u16,
        be_u32,
        be_u8,
    },
    Err,
};
use internet_checksum::Checksum;
use nom::combinator::verify;

#[derive(Debug)]
pub struct IcmpV4Header {
    message_type: u8,
    code: u8,
    remaining: u32,
}

impl IcmpV4Header {
    pub fn decode(packet: &[u8]) -> Result<(IcmpV4Header, &[u8]), Err<(&[u8], ErrorKind)>> {
        let mut checksum = Checksum::new();
        checksum.add_bytes(&packet[0..2]);
        checksum.add_bytes(&[0, 0]);
        checksum.add_bytes(&packet[4..]);

        let expected_checksum = u16::from_be_bytes(checksum.checksum());

        let state = packet;

        let (state, message_type) = be_u8(state)?;
        let (state, code) = be_u8(state)?;
        let (state, _) = verify(be_u16, |actual_checksum| *actual_checksum == expected_checksum)(state)?;
        let (state, remaining) = be_u32(state)?;

        Ok((
            IcmpV4Header {
                message_type,
                code,
                remaining,
            },
            state,
        ))
    }
}
