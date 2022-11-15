use ring::{
    aead::{Nonce, NonceSequence},
    error::Unspecified,
};

pub struct NonceSeq {
    counter: usize,
    len: usize,
}

impl NonceSeq {
    pub fn new(len: usize) -> Self {
        NonceSeq { counter: 0, len }
    }
}

impl NonceSequence for NonceSeq {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        if self.counter >= self.len {
            return Err(Unspecified);
        }

        // TODO: consider the case where counter may exceed the width of usize?
        let last_byte = (self.counter == self.len - 1) as u8;

        let mut nonce = [0u8; 12];
        nonce[3..11].copy_from_slice(&self.counter.to_be_bytes());
        nonce[11] = last_byte;

        Ok(Nonce::assume_unique_for_key(nonce))
    }
}
