use ssz::{Decode, DecodeError, Encode, SszDecoderBuilder, SszEncoder};

#[derive(Debug, PartialEq, Clone)]
pub struct Foo {
    pub a: u16,
    pub b: Vec<u8>,
    pub c: u16,
}

impl Encode for Foo {
    fn is_ssz_fixed_len() -> bool {
        <u16 as Encode>::is_ssz_fixed_len() && <Vec<u16> as Encode>::is_ssz_fixed_len()
    }

    fn ssz_bytes_len(&self) -> usize {
        <u16 as Encode>::ssz_fixed_len()
            + ssz::BYTES_PER_LENGTH_OFFSET
            + <u16 as Encode>::ssz_fixed_len()
            + self.b.ssz_bytes_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let offset = <u16 as Encode>::ssz_fixed_len()
            + <Vec<u16> as Encode>::ssz_fixed_len()
            + <u16 as Encode>::ssz_fixed_len();

        let mut encoder = SszEncoder::container(buf, offset);

        encoder.append(&self.a);
        encoder.append(&self.b);
        encoder.append(&self.c);

        encoder.finalize();
    }
}

impl Decode for Foo {
    fn is_ssz_fixed_len() -> bool {
        <u16 as Decode>::is_ssz_fixed_len() && <Vec<u16> as Decode>::is_ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut builder = SszDecoderBuilder::new(bytes);

        builder.register_type::<u16>()?;
        builder.register_type::<Vec<u8>>()?;
        builder.register_type::<u16>()?;

        let mut decoder = builder.build()?;

        Ok(Self {
            a: decoder.decode_next()?,
            b: decoder.decode_next()?,
            c: decoder.decode_next()?,
        })
    }
}
