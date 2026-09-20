pub use super::error::PacketError;

#[derive(Debug, PartialEq, Eq)]
pub struct ClientPacket {
    pub timestamp: u32,
    pub ack: u32,
    pub proto_id: u16,
    pub payload: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub struct ServerPacket {
    pub return_number: u32,
    pub proto_id: u16,
    pub payload: Vec<u8>,
}

impl ClientPacket {
    pub const HEADER_SIZE: usize = 12;
    const BODY_HEADER_SIZE: usize = Self::HEADER_SIZE - 2;

    #[allow(dead_code)]
    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        let body_size = Self::BODY_HEADER_SIZE + self.payload.len();
        let size = u16::try_from(body_size).map_err(|_| PacketError::TooLarge(body_size))?;
        let mut output = Vec::with_capacity(body_size + 2);
        output.extend_from_slice(&size.to_be_bytes());
        output.extend_from_slice(&self.timestamp.to_be_bytes());
        output.extend_from_slice(&self.ack.to_be_bytes());
        output.extend_from_slice(&self.proto_id.to_be_bytes());
        output.extend_from_slice(&self.payload);
        Ok(output)
    }

    pub fn decode(input: &[u8]) -> Result<Self, PacketError> {
        validate_length(input, Self::HEADER_SIZE, Self::BODY_HEADER_SIZE)?;
        Ok(Self {
            timestamp: u32::from_be_bytes(input[2..6].try_into().unwrap()),
            ack: u32::from_be_bytes(input[6..10].try_into().unwrap()),
            proto_id: u16::from_be_bytes(input[10..12].try_into().unwrap()),
            payload: input[Self::HEADER_SIZE..].to_vec(),
        })
    }
}

#[allow(dead_code)]
impl ServerPacket {
    pub const HEADER_SIZE: usize = 8;
    const BODY_HEADER_SIZE: usize = Self::HEADER_SIZE - 2;

    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        let body_size = Self::BODY_HEADER_SIZE + self.payload.len();
        let size = u16::try_from(body_size).map_err(|_| PacketError::TooLarge(body_size))?;
        let mut output = Vec::with_capacity(body_size + 2);
        output.extend_from_slice(&size.to_be_bytes());
        output.extend_from_slice(&self.return_number.to_be_bytes());
        output.extend_from_slice(&self.proto_id.to_be_bytes());
        output.extend_from_slice(&self.payload);
        Ok(output)
    }

    pub fn decode(input: &[u8]) -> Result<Self, PacketError> {
        validate_length(input, Self::HEADER_SIZE, Self::BODY_HEADER_SIZE)?;
        Ok(Self {
            return_number: u32::from_be_bytes(input[2..6].try_into().unwrap()),
            proto_id: u16::from_be_bytes(input[6..8].try_into().unwrap()),
            payload: input[Self::HEADER_SIZE..].to_vec(),
        })
    }
}

fn validate_length(
    input: &[u8],
    minimum_packet_size: usize,
    minimum_body_size: usize,
) -> Result<(), PacketError> {
    if input.len() < minimum_packet_size {
        return Err(PacketError::TooShort {
            minimum: minimum_packet_size,
            actual: input.len(),
        });
    }
    let body_size = u16::from_be_bytes(input[..2].try_into().unwrap()) as usize;
    if body_size < minimum_body_size {
        return Err(PacketError::InvalidBodySize {
            minimum: minimum_body_size,
            actual: body_size,
        });
    }
    let expected = body_size + 2;
    if input.len() != expected {
        return Err(PacketError::LengthMismatch {
            expected,
            actual: input.len(),
        });
    }
    Ok(())
}
