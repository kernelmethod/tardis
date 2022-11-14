//! Types and methods for serializing the compressed executable and writing
//! it out as a new binary.

use deku::prelude::*;
use lz4_flex::block::DecompressError;

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

    /// The data contained in the resource.
    #[deku(count = "length")]
    pub data: Vec<u8>,
}

impl TardisResource {
    /// Compress a block of data and store it in a [`TardisResource`] instance.
    pub fn compress(data: &[u8]) -> Self {
        let compressed_data = lz4_flex::compress_prepend_size(data);
        TardisResource {
            length: compressed_data.len(),
            data: compressed_data,
        }
    }

    /// Decompress the data block and return it.
    pub fn decompress(&self) -> Result<Vec<u8>, DecompressError> {
        lz4_flex::decompress_size_prepended(&self.data)
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
