use bytes::{Buf, BufMut};
use sszb::{
    read_offset_from_slice, sanitize_offset, DecodeError, SszDecode, SszEncode,
    BYTES_PER_LENGTH_OFFSET,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TxOpaque {
    offsets: Vec<usize>,
    bytes: Vec<u8>,
}

pub struct TransactionsOpaqueIter<'a> {
    offsets: &'a [usize],
    bytes: &'a [u8],
}
impl<'a> Iterator for TransactionsOpaqueIter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        let (offset, offsets) = self.offsets.split_first()?;
        let next_offset = offsets.first().copied().unwrap_or(self.bytes.len());
        self.offsets = offsets;
        self.bytes.get(*offset..next_offset)
    }
}

impl TxOpaque {
    pub fn iter<'a>(&'a self) -> TransactionsOpaqueIter<'a> {
        TransactionsOpaqueIter {
            offsets: &self.offsets,
            bytes: &self.bytes,
        }
    }
    fn len_offset_bytes(&self) -> usize {
        self.offsets.len().saturating_mul(BYTES_PER_LENGTH_OFFSET)
    }
}

impl SszEncode for TxOpaque {
    fn is_ssz_static() -> bool {
        false
    }

    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }

    fn ssz_max_len() -> usize {
        // max size of tx (in bytes) times max size of tx list
        1073741824 * 1048576
    }

    fn ssz_bytes_len(&self) -> usize {
        self.len_offset_bytes().saturating_add(self.bytes.len())
    }

    fn ssz_write_fixed(&self, offset: &mut usize, buf: &mut impl BufMut) {
        buf.put_slice(&offset.to_le_bytes()[0..BYTES_PER_LENGTH_OFFSET]);
        *offset += self.ssz_bytes_len();
    }

    fn ssz_write_variable(&self, buf: &mut impl BufMut) {
        self.ssz_write(buf);
    }

    fn ssz_write(&self, buf: &mut impl BufMut) {
        let len_offset_bytes = self.len_offset_bytes();
        for offset in &self.offsets {
            let offset = offset.saturating_add(len_offset_bytes);
            // buf.extend_from_slice(&encode_length(offset));
            buf.put_slice(&offset.to_le_bytes()[0..BYTES_PER_LENGTH_OFFSET]);
        }
        buf.put_slice(&self.bytes);
    }
}

impl SszDecode for TxOpaque {
    fn is_ssz_static() -> bool {
        false
    }

    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }

    fn ssz_max_len() -> usize {
        1073741824 * 1048576
    }

    fn ssz_read(
        _fixed_bytes: &mut impl Buf,
        variable_bytes: &mut impl Buf,
    ) -> Result<Self, DecodeError> {
        if !variable_bytes.has_remaining() {
            return Ok(Self::default());
        }
        let (offset_bytes, value_bytes) = {
            let first_offset =
                read_offset_from_slice(&variable_bytes.chunk()[0..BYTES_PER_LENGTH_OFFSET])?;
            if first_offset % BYTES_PER_LENGTH_OFFSET != 0 || first_offset < BYTES_PER_LENGTH_OFFSET
            {
                return Err(DecodeError::InvalidListFixedBytesLen(first_offset));
            }
            variable_bytes
                .chunk()
                .split_at_checked(first_offset)
                .ok_or(DecodeError::OffsetOutOfBounds(first_offset))?
        };
        // Disallow lists that have too many transactions.
        let num_items = offset_bytes.len() / BYTES_PER_LENGTH_OFFSET;
        let max_tx_count = 1048576;
        if num_items > max_tx_count {
            return Err(DecodeError::BytesInvalid(format!(
                "List of {} txs exceeds maximum of {:?}",
                num_items, max_tx_count
            )));
        }
        let max_tx_bytes = 1073741824;
        let mut offsets = Vec::with_capacity(num_items);
        let mut offset_iter = offset_bytes.chunks(BYTES_PER_LENGTH_OFFSET).peekable();
        while let Some(offset) = offset_iter.next() {
            let offset = read_offset_from_slice(offset)?;
            // Make the offset assume that the values start at index 0, rather
            // than following the offset bytes.
            let offset = offset
                .checked_sub(offset_bytes.len())
                .ok_or(DecodeError::OffsetIntoFixedPortion(offset))?;
            let next_offset = offset_iter
                .peek()
                .copied()
                .map(read_offset_from_slice)
                .unwrap_or(Ok(value_bytes.len()))?;
            // Disallow any offset that is lower than the previous.
            let tx_len = next_offset
                .checked_sub(offset)
                .ok_or(DecodeError::OffsetsAreDecreasing(offset))?;
            // Disallow transactions that are too large.
            if tx_len > max_tx_bytes {
                return Err(DecodeError::BytesInvalid(format!(
                    "length of {tx_len} exceeds maximum tx length of {max_tx_bytes}",
                )));
            }
            // Disallow an offset that points outside of the value bytes.
            if offset > value_bytes.len() {
                return Err(DecodeError::OffsetOutOfBounds(offset));
            }
            offsets.push(offset);
        }
        Ok(Self {
            offsets,
            bytes: value_bytes.to_vec(),
        })
    }
}
