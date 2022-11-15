//! Types and methods for serializing the compressed executable and writing
//! it out as a new binary.

use crate::crypto;
use deku::prelude::*;
use lz4_flex::block::DecompressError;
use ring::{
    aead::{self, BoundKey, UnboundKey, CHACHA20_POLY1305},
    rand,
};

/// End marker for Tardis's manifest.
///
/// The end marker helps Tardis figure out where the start of the compressed
/// executable is, so that it knows what it needs to extract into memory. The
/// end marker is placed at the end of the file so that the executable always
/// knows where to look for it.
///
/// The magic specified below is written on serialization. During deserialization,
/// Deku will check for the presence of this magic to confirm that it's reading
/// the correct data.
///
/// Note that the size of the `EndMarker` is always the magic length plus the
/// size of the fields.
#[derive(Debug, DekuRead, DekuWrite)]
#[deku(magic = b"etar")]
pub struct EndMarker {
    /// The location in the binary where the manifest starts
    pub manifest_start: usize,

    /// The number of resources in the manifest.
    pub n_resources: usize,
}

impl EndMarker {
    /// Return the number of bytes (on-disk) required to represent an `EndMarker`.
    pub const fn nbytes() -> usize {
        20
    }
}

/// Block of data that contains the compressed executable that Tardis decompresses
/// in memory.
#[derive(Debug, DekuRead, DekuWrite)]
pub struct TardisResource {
    /// The size of the resource
    #[deku(update = "self.data.len()")]
    length: usize,

    /// The encryption key for the resource
    key: [u8; 32],

    /// The data contained in the resource.
    #[deku(count = "length")]
    pub data: Vec<u8>,
}

impl TardisResource {
    /// Compress a block of data and store it in a [`TardisResource`] instance.
    pub fn compress(data: &[u8]) -> Self {
        let rng = rand::SystemRandom::new();
        let key = rand::generate::<[u8; 32]>(&rng).unwrap().expose();

        // Compress and encrypt data
        let mut data = lz4_flex::compress_prepend_size(data);
        let uk = UnboundKey::new(&CHACHA20_POLY1305, &key).unwrap();
        let nonces = crypto::NonceSeq::new(1);
        let mut sk = aead::SealingKey::new(uk, nonces);

        let aad = aead::Aad::from(b"");
        sk.seal_in_place_append_tag(aad, &mut data).unwrap();
        let len = data.len();

        TardisResource {
            key,
            data,
            length: len,
        }
    }

    /// Decompress the data block and return it.
    pub fn decompress(self) -> Result<Vec<u8>, DecompressError> {
        let uk = match UnboundKey::new(&CHACHA20_POLY1305, &self.key) {
            Ok(key) => key,
            Err(_) => panic!(),
        };
        let nonces = crypto::NonceSeq::new(1);
        let mut ok = aead::OpeningKey::new(uk, nonces);
        let aad = aead::Aad::from(b"");

        let mut data = self.data;

        let plaintext = match ok.open_in_place(aad, &mut data) {
            Ok(data) => data,
            Err(_) => panic!(),
        };
        lz4_flex::decompress_size_prepended(&plaintext)
    }

    /// Return the length of the [`TardisResource`] after it's converted to a byte
    /// string.
    pub fn len(&self) -> usize {
        8 + self.data.len()
    }

    /// Returns `true` if there isn't any data stored in the [`TardisResources`].
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::{EndMarker, TardisResource};
    use deku::DekuContainerWrite;

    #[test]
    fn test_end_marker_nbytes() {
        let marker = EndMarker {
            manifest_start: 0,
            n_resources: 16,
        };
        let marker_bytes = marker.to_bytes().unwrap();
        assert_eq!(marker_bytes.len(), EndMarker::nbytes());
    }

    #[test]
    fn test_tardis_resource_len() {
        let resource = TardisResource::compress(b"\x00\x00\x00\x00");
        let resource_bytes = resource.to_bytes().unwrap();
        assert_eq!(resource_bytes.len(), resource.len());
    }
}
