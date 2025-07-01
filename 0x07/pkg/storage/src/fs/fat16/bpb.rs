//! Fat16 BIOS Parameter Block
//!
//! reference:
//! - <https://en.wikipedia.org/wiki/BIOS_parameter_block>
//! - <https://wiki.osdev.org/FAT#Boot_Record>

/// Represents a Boot Parameter Block.
///
/// This is the first sector of a FAT 16 formatted partition,
/// and it describes various properties of the FAT 16 filesystem.
pub struct Fat16Bpb {
    data: [u8; 512],
}

impl Fat16Bpb {
    /// Attempt to parse a Boot Parameter Block from a 512 byte sector.
    pub fn new(data: &[u8]) -> Result<Fat16Bpb, &'static str> {
        let data = data.try_into().unwrap();
        let bpb = Fat16Bpb { data };

        if bpb.data.len() != 512 || bpb.trail() != 0xAA55 {
            return Err("Bad BPB format");
        }

        Ok(bpb)
    }

    pub fn total_sectors(&self) -> u32 {
        if self.total_sectors_16() == 0 {
            self.total_sectors_32()
        } else {
            self.total_sectors_16() as u32
        }
    }

    // FIXME: define all the fields in the BPB
    //      - use `define_field!` macro
    //      - ensure you can pass the tests
    //      - you may change the field names if you want
    //     对于 FAT12/16 和 FAT32，它们的 BPB 结构在首 36 字节上完全一致：

    // 域名称	偏移	大小	描述	限制
    // BS_jmpBoot	0	3	跳转到启动代码处执行的指令
    // 由于实验中启动代码位于MBR，不会从这里进行启动，因此可以不用关心这个域实际的内容	一般为0x90**EB或是0x****E9
    // BS_OEMName	3	8	OEM厂商的名称，同样与实验无关，不需要关心	-
    // BPB_BytsPerSec	11	2	每个扇区的字节数	只能是512、1024、2048或4096
    // BPB_SecPerClus	13	1	每个簇的扇区数量	只能是1、2、4、8、16、32、64和128
    // BPB_RsvdSecCnt	14	2	保留区域的扇区数量
    // 可以用来计算FAT区域的首扇区位置	不能为0，可以为任意非0值
    // 可以用来将将数据区域与簇大小对齐（使数据区域的起始偏移位于簇大小的整数倍处）
    // BPB_NumFATs	16	1	FAT表数量	一般为2，也可以为1
    // BPB_RootEntCnt	17	2	根目录中的条目数
    // 指根目录中包含的所有的条目数量，包括有效的、空的和无效的条目
    // 可以用来计算根目录区所占用的字节数	FAT32： 必须为0
    // FAT12/16： 必须满足 32 * BPB_RootEntCnt % 2 == 0
    // BPB_TotSec16	19	2	16位长度卷的总扇区数
    // 对于FAT32和更大容量的存储设备有额外的BPB_TotSec32域
    // 应当是为了维持BPB结构的一致性而仍然保留了这个域	FAT32： 必须为0
    // FAT12/16： 如果总扇区数小于0x10000（也就是能用16位表示）则使用此域表示，否则也使用BPB_TotSec32域
    // BPB_Media	21	1	似乎是设备的类型
    // 与实验无关，所以可以不用特别关心	合法取值包括0xF0、0xF8、0xF9、0xFA、0xFB、0xFC、0xFD、0xFE和0xFF
    // 本地磁盘（不可移动）的规定值为0xF8
    // 可移动磁盘的往往使用0xF0
    // BPB_FATSz16	22	2	单个FAT表占用的扇区数
    // 只用于FAT12/16格式的文件系统	FAT32： 必须为0
    // FAT12/16： 正整数值
    // BPB_SecPerTrk	24	2	每个扇区的磁道数
    // 与0x13中断相关
    // 只与具有物理结构（如磁道、磁盘等）并且对0x13中断可见的存储介质有关
    // 与实验无关，可以不用关心	-
    // BPB_NumHeads	26	2	磁头数量
    // 同样与0x13中断相关，实验不会使用，所以可以不用关心	-
    // BPB_HiddSec	28	4	分区前隐藏的扇区数
    // 在文档中描述这个域为同样只与对0x13中断可见的存储介质有关，但在实验过程中发现对于一个多分区的磁盘，这个域对应了分区首扇区在整个磁盘中的扇区号，例如首扇区位于磁盘2048扇区（从0开始计算分区号）的分区，其 BPB_HiddSec 域值就为2048	-
    // BPB_TotSec32	32	4	32位长度卷的总扇区数
    // 用来描述FAT32卷中的总扇区数或是扇区数多于0x10000的FAT12/16卷中的总扇区数	FAT32： 必须为非零整数值
    // FAT12/16： 如果扇区数大于0x10000，则为扇区数，否则必须为0
    // 从第 37 字节开始，FAT12 和 FAT16 卷上的 BPB 结构如下：

    // 域名称	偏移	大小	描述	限制
    // BS_DrvNum	36	1	用于0x13中断的驱动器号，可以不用关心	应当设置为0x80或是0x00
    // BS_Reserved1	37	1	保留位	必须为0
    // BS_BootSig	38	1	用来检验启动扇区的完整性的签名，可以不用关心	如果 BS_VolID、BS_VolLab 和 BS_FilSysType 三个域都存在有效的值 (present)，则置为0x29
    // BS_VolID	39	4	卷的序列号，可以不用关心	-
    // BS_VolLab	43	11	卷标，可以不用关心
    // 在文档中，要求与根目录下的卷标描述文件保持内容一致，但实际上在测试中往往卷标描述文件中存储的是真实的卷标而这个域的内容仍为缺省值"No NAME"	缺省值为"NO NAME"
    // BS_FilSysType	54	8	用来描述文件系统类型，但不能用来作为判断文件系统类型的依据	“FAT12”、“FAT16"或是"FAT32”
    // -	62	448	空余，置零	必须为0
    // Signature_word	510	2	校验位	设置为0xAA55
    // -	512	*	如果 BPB_BytsPerSec > 512 则存在此域，全部置零	必须为0
    define_field!([u8;8],0x03,oem_name);
    define_field!(u16,0x0b,bytes_per_sector);
    define_field!(u8,0x0d,sectors_per_cluster);
    define_field!(u16,0x0e,reserved_sector_count);
    define_field!(u8,0x10,fat_count);
    define_field!(u16,0x11,root_entries_count);
    define_field!(u16,0x13,total_sectors_16);
    define_field!(u8,0x15,media_descriptor);
    define_field!(u16,0x16,sectors_per_fat);
    define_field!(u16,0x18,sectors_per_track);
    define_field!(u16,0x1a,track_count);
    define_field!(u32,0x1c,hidden_sectors);
    define_field!(u32,0x20,total_sectors_32);
    define_field!(u8, 0x24, drive_number);
    define_field!(u8, 0x25, reserved_flags);
    define_field!(u8, 0x26, boot_signature);
    define_field!(u32, 0x27, volume_id);
    define_field!([u8; 11], 0x2b, volume_label);
    define_field!([u8; 8], 0x36, system_identifier);
    define_field!(u16, 0x1fe, trail);

}

impl core::fmt::Debug for Fat16Bpb {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Fat16 BPB")
            .field("OEM Name", &self.oem_name_str())
            .field("Bytes per Sector", &self.bytes_per_sector())
            .field("Sectors per Cluster", &self.sectors_per_cluster())
            .field("Reserved Sector Count", &self.reserved_sector_count())
            .field("FAT Count", &self.fat_count())
            .field("Root Entries Count", &self.root_entries_count())
            .field("Total Sectors", &self.total_sectors())
            .field("Media Descriptor", &self.media_descriptor())
            .field("Sectors per FAT", &self.sectors_per_fat())
            .field("Sectors per Track", &self.sectors_per_track())
            .field("Track Count", &self.track_count())
            .field("Hidden Sectors", &self.hidden_sectors())
            .field("Total Sectors", &self.total_sectors())
            .field("Drive Number", &self.drive_number())
            .field("Reserved Flags", &self.reserved_flags())
            .field("Boot Signature", &self.boot_signature())
            .field("Volume ID", &self.volume_id())
            .field("Volume Label", &self.volume_label_str())
            .field("System Identifier", &self.system_identifier_str())
            .field("Trail", &self.trail())
            .finish()
    }
}

/// Test the `Fat16Bpb` struct
///
/// WARN: do not modify following test code unless you changed the field names
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fat16_bpb_1() {
        // Taken from a Raspberry Pi bootable SD-Card
        const DATA: [u8; 192] = hex_literal::hex!(
            "EB 3C 90 6D 6B 66 73 2E 66 61 74 00 02 10 01 00
        02 00 02 00 00 F8 20 00 3F 00 FF 00 00 00 00 00
        00 E0 01 00 80 01 29 BB B0 71 77 62 6F 6F 74 20
        20 20 20 20 20 20 46 41 54 31 36 20 20 20 0E 1F
        BE 5B 7C AC 22 C0 74 0B 56 B4 0E BB 07 00 CD 10
        5E EB F0 32 E4 CD 16 CD 19 EB FE 54 68 69 73 20
        69 73 20 6E 6F 74 20 61 20 62 6F 6F 74 61 62 6C
        65 20 64 69 73 6B 2E 20 20 50 6C 65 61 73 65 20
        69 6E 73 65 72 74 20 61 20 62 6F 6F 74 61 62 6C
        65 20 66 6C 6F 70 70 79 20 61 6E 64 0D 0A 70 72
        65 73 73 20 61 6E 79 20 6B 65 79 20 74 6F 20 74
        72 79 20 61 67 61 69 6E 20 2E 2E 2E 20 0D 0A 00"
        );
        
        let mut bpb_data = Vec::with_capacity(512);
        bpb_data.extend_from_slice(&DATA);
        bpb_data.resize(510, 0u8);
        bpb_data.extend_from_slice(&[0x55, 0xAA]);

        let bpb = Fat16Bpb::new(&bpb_data).unwrap();

        assert_eq!(bpb.oem_name(), b"mkfs.fat");
        assert_eq!(bpb.bytes_per_sector(), 512);
        assert_eq!(bpb.sectors_per_cluster(), 16);
        assert_eq!(bpb.reserved_sector_count(), 1);
        assert_eq!(bpb.fat_count(), 2);
        assert_eq!(bpb.root_entries_count(), 512);
        assert_eq!(bpb.total_sectors_16(), 0);
        assert_eq!(bpb.media_descriptor(), 0xf8);
        assert_eq!(bpb.sectors_per_fat(), 32);
        assert_eq!(bpb.sectors_per_track(), 63);
        assert_eq!(bpb.track_count(), 255);
        assert_eq!(bpb.hidden_sectors(), 0);
        assert_eq!(bpb.total_sectors_32(), 0x1e000);
        assert_eq!(bpb.drive_number(), 128);
        assert_eq!(bpb.reserved_flags(), 1);
        assert_eq!(bpb.boot_signature(), 0x29);
        assert_eq!(bpb.volume_id(), 0x7771b0bb);
        assert_eq!(bpb.volume_label(), b"boot       ");
        assert_eq!(bpb.system_identifier(), b"FAT16   ");

        assert_eq!(bpb.total_sectors(), 0x1e000);

        println!("{:#?}", bpb);
    }

    #[test]
    fn test_fat16_bpb_2() {
        // Taken from QEMU VVFAT
        const DATA: [u8; 64] = hex_literal::hex!(
            "EB 3E 90 4D 53 57 49 4E 34 2E 31 00 02 10 01 00
        02 00 02 00 00 F8 FC 00 3F 00 10 00 3F 00 00 00
        C1 BF 0F 00 80 00 29 FD 1A BE FA 51 45 4D 55 20
        56 56 46 41 54 20 46 41 54 31 36 20 20 20 00 00"
        );

        let mut bpb_data = Vec::with_capacity(512);
        bpb_data.extend_from_slice(&DATA);
        bpb_data.resize(510, 0u8);
        bpb_data.extend_from_slice(&[0x55, 0xAA]);

        let bpb = Fat16Bpb::new(&bpb_data).unwrap();

        assert_eq!(bpb.oem_name(), b"MSWIN4.1");
        assert_eq!(bpb.oem_name_str(), "MSWIN4.1");
        assert_eq!(bpb.bytes_per_sector(), 512);
        assert_eq!(bpb.sectors_per_cluster(), 16);
        assert_eq!(bpb.reserved_sector_count(), 1);
        assert_eq!(bpb.fat_count(), 2);
        assert_eq!(bpb.root_entries_count(), 512);
        assert_eq!(bpb.total_sectors_16(), 0);
        assert_eq!(bpb.media_descriptor(), 0xf8);
        assert_eq!(bpb.sectors_per_fat(), 0xfc);
        assert_eq!(bpb.sectors_per_track(), 63);
        assert_eq!(bpb.track_count(), 16);
        assert_eq!(bpb.hidden_sectors(), 63);
        assert_eq!(bpb.total_sectors_32(), 0xfbfc1);
        assert_eq!(bpb.drive_number(), 128);
        assert_eq!(bpb.reserved_flags(), 0);
        assert_eq!(bpb.boot_signature(), 0x29);
        assert_eq!(bpb.volume_id(), 0xfabe1afd);
        assert_eq!(bpb.volume_label(), b"QEMU VVFAT ");
        assert_eq!(bpb.volume_label_str(), "QEMU VVFAT ");
        assert_eq!(bpb.system_identifier(), b"FAT16   ");
        assert_eq!(bpb.system_identifier_str(), "FAT16   ");

        assert_eq!(bpb.total_sectors(), 0xfbfc1);

        println!("{:#?}", bpb);
    }
}