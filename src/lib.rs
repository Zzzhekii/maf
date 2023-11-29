use std::collections::hash_map::Entry;

#[no_main]
#[no_std]

const MAGIC_VALUE: [u8; 9] = [0xBA, 0xDA, 0x55, 0x6D, 0x61, 0x66, 0x67, 0x65, 0x78];

#[derive(Debug)]
pub struct Archive<'a> {
    pub header: ArchiveHeader,
    pub entry_mapping_list: Vec<EntryMapping>,
    pub entry_path_list: Vec<&'a [u8]>,
    pub entry_data_list: Vec<&'a [u8]>,
}

impl<'a> Archive<'a> {
    pub fn from_raw_bytes(bytes: &'a [u8]) -> Result<Self, ()> {
        unsafe fn bytes_to_struct<T: Sized>(bytes: &[u8]) -> Result<&T, ()> {
            let (head, body, _tail) = unsafe { bytes.align_to::<T>() };
            // if head.is_empty() {
            //     return Err(())
            // }
            Ok(&body[0])
        }

        const HEADER_SIZE: usize = core::mem::size_of::<ArchiveHeader>();
        const ENTRY_MAPPING_SIZE: usize = core::mem::size_of::<EntryMapping>();

        let header = (*unsafe { bytes_to_struct::<ArchiveHeader>(&bytes[0..HEADER_SIZE]) }?).clone();

        let entry_mapping_list_offset = HEADER_SIZE + header.mime_length as usize;
        let mut entry_mapping_list: Vec<EntryMapping> = Vec::new();
        for i in 0..header.entry_amount as usize {
            let offset = entry_mapping_list_offset + i * ENTRY_MAPPING_SIZE;
            let entry_mapping = unsafe {
                bytes_to_struct::<EntryMapping>(&bytes[offset..offset + ENTRY_MAPPING_SIZE])
            }?;
            entry_mapping_list.push((*entry_mapping).clone());
        }

        let entry_path_list_offset =
            entry_mapping_list_offset + header.entry_amount as usize * ENTRY_MAPPING_SIZE;
        let mut entry_path_list: Vec<&[u8]> = Vec::new();
        for i in 0..header.entry_amount as usize {
            let offset = entry_path_list_offset + entry_mapping_list[i].entry_path_offset as usize;
            let length = if let Some(mapping) = entry_mapping_list.get(i + 1) {
                mapping.entry_path_offset
            } else {
                header.entry_path_list_size
            } - entry_mapping_list[i].entry_path_offset;
            let entry_path = &bytes[offset..length as usize + offset];
            entry_path_list.push(entry_path);
        }

        let entry_data_list_offset = entry_path_list_offset + header.entry_path_list_size as usize;
        let mut entry_data_list: Vec<&[u8]> = Vec::new();
        for i in 0..header.entry_amount as usize  {
            let offset = entry_data_list_offset + entry_mapping_list[i].entry_data_offset as usize;
            let length = if let Some(mapping) = entry_mapping_list.get(i + 1) {
                mapping.entry_data_offset
            } else {
                header.entry_data_list_size
            } - entry_mapping_list[i].entry_data_offset;
            let entry_data = &bytes[offset..length as usize + offset];
            entry_data_list.push(entry_data)
        }

        Ok(Self {
            header,
            entry_mapping_list,
            entry_path_list,
            entry_data_list,
        })
    }

    pub fn to_raw_bytes(&self) -> Vec<u8> {
        unsafe fn struct_to_bytes<T: Sized>(p: &T) -> &[u8] {
            core::slice::from_raw_parts::<u8>(
                (p as *const T) as *const u8,
                core::mem::size_of::<T>(),
            )
        }

        let mut result = unsafe { struct_to_bytes(&self.header) }.to_vec();

        result.append(
            &mut self
                .entry_mapping_list
                .iter()
                .map(|e| unsafe { struct_to_bytes(e) })
                .collect::<Vec<&[u8]>>()
                .concat(),
        );

        result.append(&mut self.entry_path_list.concat());
        result.append(&mut self.entry_data_list.concat());

        result
    }

    /// Returns Vec<(path, data)>
    pub fn list_entries(&self) -> Vec<(&[u8], &[u8])> {
        let mut entries: Vec<(&[u8], &[u8])> = Vec::new();
        for i in 0..self.entry_mapping_list.len() {
            entries.push((
                self.entry_path_list[i],
                self.entry_data_list[i],
            ))
        }
        entries
    }
}

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct ArchiveHeader {
    pub magic_value: [u8; 9],
    pub version_number: u8,
    pub mime_length: u16,
    pub entry_amount: u32,
    pub entry_path_list_size: u64,
    pub entry_data_list_size: u128,
    pub _reserved: [u8; 24],
}

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct EntryMapping {
    pub entry_path_offset: u64,
    pub entry_data_offset: u128,
    pub _reserved: u64,
}

#[derive(Debug)]
pub struct Path<'a>(pub &'a [u8]);

pub struct Builder<'a> {
    entries: Vec<(Path<'a>, &'a [u8])>,
}

impl<'a> Builder<'a> {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn build(&self) -> Archive {
        let mut entry_mapping_list: Vec<EntryMapping> = Vec::new();
        let mut entry_path_list: Vec<&[u8]> = Vec::new();
        let mut entry_data_list: Vec<&[u8]> = Vec::new();

        let mut entry_data_list_size: u128 = 0;
        let mut entry_path_list_size: u64 = 0;

        for (path, data) in &self.entries {
            let entry_index = entry_mapping_list.len();

            let path = path.0;

            entry_mapping_list.push(EntryMapping {
                entry_path_offset: entry_path_list_size,
                entry_data_offset: entry_data_list_size,
                _reserved: 0,
            });

            entry_data_list.push(data);
            entry_path_list.push(path);

            entry_data_list_size += data.len() as u128;
            entry_path_list_size += path.len() as u64;
        }

        let header = ArchiveHeader {
            magic_value: MAGIC_VALUE,
            version_number: 0,
            mime_length: 0,
            entry_amount: self.entries.len() as u32,
            entry_path_list_size,
            entry_data_list_size,
            _reserved: [0u8; 24],
        };

        let mut archive = Archive {
            header,
            entry_mapping_list,
            entry_path_list,
            entry_data_list,
        };

        archive
    }

    pub fn add_entry(&mut self, path: Path<'a>, data: &'a [u8]) -> &mut Self {
        self.entries.push((path, data));
        self
    }
}
