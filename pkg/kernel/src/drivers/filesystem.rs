use super::ata::*;
use alloc::boxed::Box;
use chrono::DateTime;
use storage::fat16::Fat16;
use storage::mbr::*;
use storage::*;
use alloc::format;
use crate::alloc::string::ToString;
pub static ROOTFS: spin::Once<Mount> = spin::Once::new();

pub fn get_rootfs() -> &'static Mount {
    ROOTFS.get().unwrap()
}

pub fn init() {
    info!("Opening disk device...");

    let drive = AtaDrive::open(0, 0).expect("Failed to open disk device");

    // only get the first partition
    let part = MbrTable::parse(drive)
        .expect("Failed to parse MBR")
        .partitions()
        .expect("Failed to get partitions")
        .remove(0);

    info!("Mounting filesystem...");

    ROOTFS.call_once(|| Mount::new(Box::new(Fat16::new(part)), "/".into()));

    trace!("Root filesystem: {:#?}", ROOTFS.get().unwrap());

    info!("Initialized Filesystem.");
}

pub fn ls(root_path: &str) {
    let iter = match get_rootfs().read_dir(root_path) {
        Ok(iter) => iter,
        Err(err) => {
            warn!("{:?}", err);
            return;
        }
    };

    // FIXME: format and print the file metadata
    //      - use `for meta in iter` to iterate over the entries
    //      - use `crate::humanized_size_short` for file size
    //      - add '/' to the end of directory names
    //      - format the date as you like
    //      - do not forget to print the table header
    println!(
        "{:<12} {:<12} {:<12} {:<20} {:<20} {:<20}",
        "Name", "Type", "Size", "Created Time", "Last Modified", "Last Access"
    );
    for meta in iter {
        let filetype = {
            if meta.is_dir() {
                "Directory"
            } else {
                "File"
            }
        };
        let mut name = meta.name;
        if filetype == "Directory" {
            name.push('/');
        }
        let (num, unit) = crate::humanized_size(meta.len as u64);
        let size = format!("{:.2} {}", num, unit);
        let created_time = meta
            .created
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let last_modified = meta
            .modified
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let last_access = meta
            .accessed
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        println!(
            "{:<12} {:<12} {:<12} {:<20} {:<20} {:<20}",
            name, filetype, size, created_time, last_modified, last_access
        );
    }
}
