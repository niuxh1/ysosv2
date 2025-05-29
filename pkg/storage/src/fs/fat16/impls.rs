

use core::f32::consts::E;

use super::*;

impl Fat16Impl {
    pub fn new(inner: impl BlockDevice<Block512>) -> Self {
        let mut block = Block::default();
        let block_size = Block512::size();

        inner.read_block(0, &mut block).unwrap();
        let bpb = Fat16Bpb::new(block.as_ref()).unwrap();

        trace!("Loading Fat16 Volume: {:#?}", bpb);

        // HINT: FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz) + RootDirSectors;
        let fat_start = bpb.reserved_sector_count() as usize;
        let root_dir_size =  bpb.root_entries_count() as usize * 32 / block_size;
        let first_root_dir_sector = fat_start + (bpb.fat_count() as usize * bpb.sectors_per_fat() as usize);
        let first_data_sector = first_root_dir_sector + root_dir_size;

        Self {
            bpb,
            inner: Box::new(inner),
            fat_start,
            first_data_sector,
            first_root_dir_sector,
        }
    }

    pub fn cluster_to_sector(&self, cluster: &Cluster) -> usize {
        match *cluster {
            Cluster::ROOT_DIR => self.first_root_dir_sector,
            Cluster(c) => {
                // FIXME: calculate the first sector of the cluster
                // HINT: FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                return ((c-2)*self.bpb.sectors_per_cluster() as u32) as usize+ self.first_data_sector ;
            }
        }
    }

    // FIXME: YOU NEED TO IMPLEMENT THE FILE SYSTEM OPERATIONS HERE
    //      - read the FAT and get next cluster

    pub fn get_next_cluster(&self, cluster: &Cluster) -> Result<Cluster> {
        if *cluster == Cluster::ROOT_DIR {
            Ok(Cluster::END_OF_FILE)
        } else {
            let fat_offset = (cluster.0 * 2) as usize;
            let sector = self.fat_start + (fat_offset / BLOCK_SIZE);
            let offset_in_sector = fat_offset % BLOCK_SIZE;

            let mut block: Block<512> = Block::default();
            self.inner.read_block(sector, &mut block)?;
            let next = u16::from_le_bytes(
                block[offset_in_sector..offset_in_sector + 2]
                    .try_into()
                    .map_err(|_| FsError::InvalidOperation)?
            ) as u32;

            match Cluster(next) {
                Cluster::EMPTY => Ok(Cluster::EMPTY),
                Cluster::BAD => Err(FsError::BadCluster),
                Cluster(c) => {
                    if (0x0000_0002..0x0000_FFF6).contains(&c) {
                        Ok(Cluster(c))
                    } else if c >= 0xFFFF_FFF8 {
                        Ok(Cluster::END_OF_FILE)
                    } else {
                        Ok(Cluster::INVALID)
                    }
                }
            }
        }
    }

    //      - traverse the cluster chain and read the data
    pub fn traverse_cluster_chain(&self, dir: &Directory) -> Result<Vec<Metadata>> {
        let mut entries = Vec::new();
        let mut current_cluster = dir.cluster;
        let mut block: Block<512> = Block::default(); 
        let entries_per_sector = BLOCK_SIZE / DirEntry::LEN;

        loop {

            let sectors_to_read_in_cluster = match current_cluster {
                Cluster::ROOT_DIR => {
                    // 根目录区域有固定的大小
                    // BPB 中的 root_dir_entries 指定了32字节条目的数量
                    let root_dir_bytes = self.bpb.root_entries_count() as usize * DirEntry::LEN;

                    (root_dir_bytes + BLOCK_SIZE - 1) / BLOCK_SIZE
                }
                _ => {

                    self.bpb.sectors_per_cluster() as usize
                }
            };

            let first_sector_of_this_cluster = self.cluster_to_sector(&current_cluster);

            for sector_idx_in_cluster in 0..sectors_to_read_in_cluster {
                let current_sector_lba = first_sector_of_this_cluster + sector_idx_in_cluster;
                self.inner.read_block(current_sector_lba, &mut block)?;

                for entry_idx_in_sector in 0..entries_per_sector {
                    let entry_offset_in_block = entry_idx_in_sector * DirEntry::LEN;

                    if entry_offset_in_block + DirEntry::LEN > block.len() {

                        break; 
                    }
                    let entry_slice = &block[entry_offset_in_block .. entry_offset_in_block + DirEntry::LEN];

                    match DirEntry::parse(entry_slice) {
                        Ok(dir_entry) => {
                            if dir_entry.filename.is_eod() {

                                return Ok(entries);
                            }



                            if !dir_entry.is_long_name()  {entries.push(dir_entry.as_meta());}
                        }
                        Err(_parse_err) => {

                            continue;
                        }
                    }
                }
            }

            if current_cluster == Cluster::ROOT_DIR {
                break; 
            }

            match self.get_next_cluster(&current_cluster) {
                Ok(next_cluster) => {
                    if next_cluster == Cluster::END_OF_FILE { 
                        break; 
                    }
                    current_cluster = next_cluster;
                }
                Err(e) => {

                    warn!("Error getting next cluster: {:?}", e);
                    return Err(e.into());
                }
            }
        }
        Ok(entries)
    }
    
    //      - parse the path
    pub fn match_name_entry(&self,name:&ShortFileName,sector:usize)->Result<DirEntry>{
        let mut block: Block<512> = Block::default();
        self.inner.read_block(sector, &mut block);
        for entry in 0..(BLOCK_SIZE / DirEntry::LEN) {
            let dir_entry = DirEntry::parse(&block[entry * DirEntry::LEN..(entry + 1) * DirEntry::LEN])
                                        .map_err(|_| FsError::InvalidOperation)?;
            if dir_entry.filename.is_eod() {
                return Err(FsError::FileNotFound);
            }
            if dir_entry.filename.matches(name) {
                return Ok(dir_entry);
            }
        }
        Err(FsError::NotInSector)
    }

    //      - open the root directory
    pub fn open_directory(&self,dir:&Directory,name:&str) -> Result<DirEntry>{
        let match_name =ShortFileName::parse(&name)?;
        let mut current_cluster = dir.cluster;
        let mut block: Block<512> = Block::default();
        let entries_per_sector = BLOCK_SIZE / DirEntry::LEN;
        let  sector_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        let mut sector_offset = self.cluster_to_sector(&current_cluster);
        loop{
            for sector in sector_offset..sector_offset + sector_size {
                match self.match_name_entry(&match_name, sector) {
                    Err(FsError::NotInSector) => continue,
                    x => return x,
                }
            }
            current_cluster = if current_cluster != Cluster::ROOT_DIR {
                match self.get_next_cluster(&current_cluster) {
                    Ok(n) => {
                        sector_offset = self.cluster_to_sector(&n);
                        n
                    }
                    _ => break,
                }
            } else {
                break
            }
        }
        Err(FsError::FileNotFound)


    }
    pub fn get_dir_from_name(&self,path: &str)->Result<Directory>{
       let mut path = path.split(PATH_SEPARATOR);
        let mut current = Directory::root();

        while let Some(dir) = path.next() {
            if dir.is_empty() {
                continue;
            }

            let entry = self.open_directory(&current, dir)?;

            if entry.is_directory() {
                current = Directory::from_entry(entry);
            } else if path.next().is_some() {
                return Err(FsError::NotADirectory);
            } else {
                break;
            }
        }

        Ok(current)
    }
    //      - ...
    //      - finally, implement the FileSystem trait for Fat16 with `self.handle`

}

impl FileSystem for Fat16 {
    fn read_dir(&self, path: &str) -> Result<Box<dyn Iterator<Item = Metadata> + Send>> {
        // FIXME: read dir and return an iterator for all entries
        let dir = self.handle.get_dir_from_name(path)?;

        let  entries = self.handle.traverse_cluster_chain(&dir)?;

        Ok(Box::new(entries.into_iter()))
    }

    fn open_file(&self, path: &str) -> Result<FileHandle> {
        // FIXME: open file and return a file handle
        let parent = self.handle.get_dir_from_name(path)?;
        let name = path.rsplit(PATH_SEPARATOR).next().unwrap_or("");
        let entry = self.handle.open_directory(&parent, name)?;
        if entry.is_directory() {
            return Err(FsError::NotAFile);
        }

        let handle = self.handle.clone();
        let meta = entry.as_meta();
        let file = Box::new(File::new(handle, entry));

        let file_handle = FileHandle::new(meta, file);

        Ok(file_handle)
    }

    fn metadata(&self, path: &str) -> Result<Metadata> {
        // FIXME: read metadata of the file / dir
        let parent = self.handle.get_dir_from_name(path)?;
        let name = path.rsplit(PATH_SEPARATOR).next().unwrap_or("");
        let entry = self.handle.open_directory(&parent, name)?;
        Ok(entry.as_meta())
    }

    fn exists(&self, path: &str) -> Result<bool> {
        // FIXME: check if the file / dir exists
        let parent = self.handle.get_dir_from_name(path)?;
        let name = path.rsplit(PATH_SEPARATOR).next().unwrap_or("");
        let entry = self.handle.open_directory(&parent, name);
        Ok(entry.is_ok())
    }
}
