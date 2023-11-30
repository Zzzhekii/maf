use std::io::Read;

const MAF_MAGIC_VALUE: [u8; 9] = [0xBA, 0xDA, 0x55, 0x6D, 0x61, 0x66, 0x67, 0x65, 0x78];

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Path {
    path: String,
}

impl Path {
    fn is_legal_path_char(ch: char) -> bool {
        if ch.is_ascii_alphanumeric() || ch.is_ascii_punctuation() || ch == ' ' {
            return true;
        }
        false
    }

    fn is_legal_path_str(path: &str) -> bool {
        for ch in path.chars() {
            if !Self::is_legal_path_char(ch) {
                return false;
            }
        }
        true
    }

    pub fn from_maf_str(path: &str) -> Result<Self, Error> {
        if !Self::is_legal_path_str(path) {
            return Err(Error::IllegalPath {
                path: Box::new(path.to_string()),
            });
        }
        Ok(Self {
            path: path.to_string(),
        })
    }

    pub fn from_unix_str(path: &str) -> Result<Self, Error> {
        if !Self::is_legal_path_str(path) {
            return Err(Error::IllegalPath {
                path: Box::new(path.to_string()),
            });
        }
        Ok(Self {
            path: path.to_string(),
        })
    }

    pub fn path(&self) -> &str {
        return &self.path;
    }
}

pub struct Archive {
    entries: Vec<Entry>,
}

impl Archive {
    pub fn builder<R: Read>() -> ArchiveBuilder<R> {
        ArchiveBuilder::new()
    }

    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    pub fn read<R: Read>(source: &mut R) -> Result<Self, Error> {
        // Read the source.
        let mut bytes: Vec<u8> = Vec::new();
        source.read_to_end(&mut bytes).unwrap();

        // Read the header
        let Ok(header) =
            (unsafe { bytes_to_struct::<Header>(&bytes[0..std::mem::size_of::<Header>()]) })
        else {
            return Err(Error::HeaderReadError);
        };

        // Check the magic value.
        if header.magic_value != MAF_MAGIC_VALUE {
            return Err(Error::WrongMagicValue);
        }

        // Calculate the offsets.
        let data_list_offset = std::mem::size_of::<Header>();
        let mime_offset = data_list_offset + header.entry_data_list_size as usize;
        let mapping_list_offset = mime_offset + header.mime_length as usize;
        let path_list_offset = mapping_list_offset
            + header.entry_amount as usize * std::mem::size_of::<EntryMapping>();

        // Read the mappings.
        let mut mappings: Vec<&EntryMapping> = Vec::new();
        for i in 0..header.entry_amount as usize {
            let mapping_offset = mapping_list_offset + i * std::mem::size_of::<EntryMapping>();

            let Ok(mapping) = (unsafe {
                bytes_to_struct::<EntryMapping>(
                    &bytes[mapping_offset..mapping_offset + std::mem::size_of::<EntryMapping>()],
                )
            }) else {
                return Err(Error::MappingReadError { index: i });
            };

            mappings.push(mapping);
        }

        // Read the lists.
        let mut entries: Vec<Entry> = Vec::new();
        for (i, mapping) in mappings.iter().enumerate() {
            // Calculate ending offsets.
            let (path_end_offset, data_end_offset) = if let Some(mapping) = mappings.get(i + 1) {
                (
                    path_list_offset + mapping.entry_path_offset as usize,
                    data_list_offset + mapping.entry_data_offset as usize,
                )
            } else {
                (
                    path_list_offset + header.entry_path_list_size as usize,
                    data_list_offset + header.entry_data_list_size as usize,
                )
            };

            let Ok(path) = String::from_utf8(
                bytes[path_list_offset + mapping.entry_path_offset as usize..path_end_offset]
                    .to_vec(),
            ) else {
                return Err(Error::PathReadError { index: i });
            };

            entries.push(Entry {
                path: Path::from_maf_str(&path).unwrap(),
                contents: bytes[data_list_offset + mapping.entry_data_offset as usize..data_end_offset].to_vec(),
            });
        }

        Ok(Self { entries })
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut header = Header {
            magic_value: MAF_MAGIC_VALUE,
            version_number: 0,
            mime_length: 0,
            entry_amount: self.entries.len() as u32,
            entry_path_list_size: 0,
            entry_data_list_size: 0,
            _reserved: [0u8; 24],
        };

        let mut entry_paths: Vec<u8> = Vec::new();
        let mut entry_mappings: Vec<u8> = Vec::new();
        let mut entry_data: Vec<u8> = Vec::new();

        // Populate all the lists, and update the header.
        for mut entry in self.entries {
            let mut path = entry.path.path().as_bytes().to_vec();
            let path_length = path.len();
            let contents_length = entry.contents.len();

            // Add the path and the data.
            entry_paths.append(&mut path);
            entry_data.append(&mut entry.contents);

            // Add the mapping.
            let mapping = EntryMapping {
                entry_path_offset: header.entry_path_list_size,
                entry_data_offset: header.entry_data_list_size,
                _reserved: 0,
            };

            entry_mappings
                .append(&mut unsafe { struct_to_bytes::<EntryMapping>(&mapping) }.to_vec());

            // Update the size fields in the header
            header.entry_path_list_size += path_length as u64;
            header.entry_data_list_size += contents_length as u128;
        }

        [
            unsafe { struct_to_bytes::<Header>(&header) },
            &entry_data,
            &entry_mappings,
            &entry_paths,
        ]
        .concat()
    }
}

#[derive(Debug)]
pub struct Entry {
    pub path: Path,
    pub contents: Vec<u8>,
}

#[derive(Debug)]
#[repr(C, packed)]
struct Header {
    pub magic_value: [u8; 9],
    pub version_number: u8,
    pub mime_length: u16,
    pub entry_amount: u32,
    pub entry_path_list_size: u64,
    pub entry_data_list_size: u128,
    pub _reserved: [u8; 24],
}

#[derive(Debug)]
#[repr(C, packed)]
struct EntryMapping {
    pub entry_path_offset: u64,
    pub entry_data_offset: u128,
    pub _reserved: u64,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Illegal path was provided: `{path:?}`")]
    IllegalPath { path: Box<dyn std::fmt::Debug> },
    #[error("Couldn't read the archive's header")]
    HeaderReadError,
    #[error("Wrong magic value in the archive's header")]
    WrongMagicValue,
    #[error("Couldn't read an entry's mapping by index `{index}`")]
    MappingReadError { index: usize },
    #[error("Couldn't read an entry's path by index `{index}`")]
    PathReadError { index: usize },
}

pub struct ArchiveBuilder<R: Read> {
    entries: Vec<(Path, R)>,
}

impl<R: Read> ArchiveBuilder<R> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, path: Path, contents: R) -> &mut Self {
        self.entries.push((path, contents));
        self
    }

    pub fn build(mut self) -> Archive {
        let entries: Vec<Entry> = self
            .entries
            .iter_mut()
            .map(|(path, read_contents)| {
                let mut contents: Vec<u8> = Vec::new();
                read_contents.read_to_end(&mut contents).unwrap();

                Entry {
                    path: path.clone(),
                    contents,
                }
            })
            .collect();

        Archive { entries }
    }
}

unsafe fn bytes_to_struct<T: Sized>(bytes: &[u8]) -> Result<&T, ()> {
    // TODO: ERROR REPORTING
    let (_head, body, _tail) = unsafe { bytes.align_to::<T>() };
    // if head.is_empty() {
    //     return Err(());
    // }
    Ok(&body[0])
}

unsafe fn struct_to_bytes<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts::<u8>((p as *const T) as *const u8, core::mem::size_of::<T>())
}
