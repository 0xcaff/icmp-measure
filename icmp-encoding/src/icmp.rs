use internet_checksum::Checksum;
use nom::{
    combinator::verify,
    error::ErrorKind,
    number::complete::{
        be_u16,
        be_u32,
        be_u8,
    },
    Err,
};

#[derive(PartialEq, Debug)]
pub struct IcmpV4Header {
    pub message_type: u8,
    pub code: u8,
    pub remaining: u32,
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
        let (state, _) = verify(be_u16, |actual_checksum| {
            *actual_checksum == expected_checksum
        })(state)?;
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

    pub fn encode(&self, buffer: &mut WritableIcmpPacket) {
        // Write Header
        {
            let header = buffer.header();
            header[0] = self.message_type;
            header[1] = self.code;

            header[4..8].clone_from_slice(&self.remaining.to_be_bytes());
        }

        // Checksum
        let mut checksum = Checksum::new();
        checksum.add_bytes(&buffer.full()[0..2]);
        checksum.add_bytes(&[0, 0]);
        checksum.add_bytes(&buffer.full()[4..]);

        let calculated_checksum = checksum.checksum();

        // Insert Checksum
        {
            let header = buffer.header();
            header[2..4].clone_from_slice(&calculated_checksum);
        }
    }
}

pub struct WritableIcmpPacket<'a> {
    buffer: &'a mut [u8],
}

impl WritableIcmpPacket<'_> {
    pub fn new(buffer: &mut [u8]) -> Option<WritableIcmpPacket> {
        if buffer.len() < 8 {
            None
        } else {
            Some(WritableIcmpPacket { buffer })
        }
    }

    pub fn header(&mut self) -> &mut [u8] {
        &mut self.buffer[0..8]
    }

    pub fn data(&mut self) -> &mut [u8] {
        &mut self.buffer[8..]
    }

    pub fn full(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

#[cfg(test)]
mod test {
    use crate::{
        icmp::WritableIcmpPacket,
        IcmpV4Header,
    };

    fn encoded() -> [u8; 64] {
        [
            0, 0, 131, 227, 32, 5, 0, 15, 94, 165, 50, 160, 0, 0, 223, 191, 8, 9, 10, 11, 12, 13,
            14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
            36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
        ]
    }

    fn decoded() -> IcmpV4Header {
        IcmpV4Header {
            message_type: 0,
            code: 0,
            remaining: 537198607,
        }
    }

    #[test]
    fn decode() {
        let encoded = &encoded()[..];
        let (header, body) = IcmpV4Header::decode(encoded).unwrap();

        assert_eq!((header, body), (decoded(), &encoded[8..]))
    }

    #[test]
    fn encode() {
        let decoded = decoded();

        let mut output_buffer = [0u8; 64];
        let encoded = &encoded()[..];

        let mut wrapped_output_buffer = WritableIcmpPacket::new(&mut output_buffer).unwrap();
        wrapped_output_buffer.data().clone_from_slice(&encoded[8..]);

        decoded.encode(&mut wrapped_output_buffer);

        assert_eq!(encoded, wrapped_output_buffer.full(),)
    }
}
