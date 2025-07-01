//! File
//!
//! reference: <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>

use core::cmp::min;
use super::*;

#[derive(Debug, Clone)]
pub struct File {
    /// The current offset in the file
    offset: usize,
    /// The current cluster of this file
    current_cluster: Cluster,
    /// DirEntry of this file
    entry: DirEntry,
    /// The file system handle that contains this file
    handle: Fat16Handle,
}

impl File {
    pub fn new(handle: Fat16Handle, entry: DirEntry) -> Self {
        Self {
            offset: 0,
            current_cluster: entry.cluster,
            entry,
            handle,
        }
    }

    pub fn length(&self) -> usize {
        self.entry.size as usize
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // FIXME: read file content from disk
        //      CAUTION: file length / buffer size / offset
        //
        //      - `self.offset` is the current offset in the file in bytes
        //      - use `self.handle` to read the blocks
        //      - use `self.entry` to get the file's cluster
        //      - use `self.handle.cluster_to_sector` to convert cluster to sector
        //      - update `self.offset` after reading
        //      - update `self.cluster` with FAT if necessary
        let bytes_per_sec = self.handle.bpb.bytes_per_sector();
        let sec_per_clus = self.handle.bpb.sectors_per_cluster();
        let mut block = Block::default();
        let mut sector_num = self.offset / bytes_per_sec as usize; 
        let mut bytes_num = self.offset % bytes_per_sec as usize;
        let mut current_offset = self.handle.cluster_to_sector(&self.current_cluster) + sector_num;
        let mut space = buf.len();
        let mut read:usize = 0;
        while  space > 0 || self.offset == self.entry.size as usize {
            self.handle.inner.read_block(current_offset,&mut block)?;
            let need_to_read = min(space,
                                        min(bytes_per_sec as usize - bytes_num, 
                                                self.entry.size as usize - self.offset)
                                            );
            space -= need_to_read;
            self.offset += need_to_read;
            buf[read .. read+need_to_read].copy_from_slice(&block[bytes_num .. bytes_num+need_to_read]);
            read += need_to_read;
            bytes_num = 0;
            sector_num = match sector_num+1 {
                sec_per_clus=> {
                    self.current_cluster = self.handle.get_next_cluster(&self.current_cluster)?;
                    0
                }
                _ => sector_num + 1,
            }
            
        }
        return Ok(read);
    }
}

// NOTE: `Seek` trait is not required for this lab
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize> {
        unimplemented!()
    }
}

// NOTE: `Write` trait is not required for this lab
impl Write for File {
    fn write(&mut self, _buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> Result<()> {
        unimplemented!()
    }
}
