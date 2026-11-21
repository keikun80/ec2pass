//! Pure-Rust ISO9660 + Rock Ridge (`cidata`) image builder for cloud-init NoCloud seeds.
//!
//! Cloud images (including Amazon Linux 2023 KVM qcow2 images and Ubuntu Cloud images)
//! look for a block device or CD-ROM with volume label `cidata` (or `CIDATA`) containing:
//! - `meta-data`
//! - `user-data`
//! - `vendor-data` (optional)
//! - `network-config` (optional)
//!
//! Because ISO9660 Level 1 normally restricts filenames to uppercase 8.3 characters without
//! hyphens, we emit both:
//! 1. Relaxed ISO9660 Level 3 / lowercase ASCII identifiers AND
//! 2. Standard IEEE P1282 Rock Ridge (`SP`, `RR`, `NM`, `PX`) System Use entries in the root
//!    directory records so the Linux kernel `isofs` driver always mounts exact lowercase
//!    hyphenated filenames (`meta-data`, `user-data`, `vendor-data`, `network-config`).

use anyhow::{bail, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

const SECTOR_SIZE: usize = 2048;

/// Represents an in-memory file to be written into the root directory of the ISO9660 image.
#[derive(Debug, Clone)]
pub struct IsoFile {
    pub name: String,
    pub content: Vec<u8>,
}

impl IsoFile {
    pub fn new(name: impl Into<String>, content: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }
}

/// Builds a complete ISO9660 image with Rock Ridge extensions (`NM`, `PX`) and volume ID `cidata`.
pub fn build_cidata_iso(volume_label: &str, files: &[IsoFile]) -> Result<Vec<u8>> {
    if files.is_empty() {
        bail!("ISO image must contain at least one file");
    }

    // Sector layout:
    // 0..16  : System Area (16 zeroed sectors = 32 KiB)
    // 16     : Primary Volume Descriptor (PVD)
    // 17     : Volume Descriptor Set Terminator (VDST)
    // 18     : Little-Endian Path Table (L-Table)
    // 19     : Big-Endian Path Table (M-Table)
    // 20     : Root Directory Extent (1 sector)
    // 21..N  : File Data Extents
    let root_dir_lba: u32 = 20;
    let mut current_lba: u32 = 21;

    struct PlannedFile<'a> {
        file: &'a IsoFile,
        iso_ident: String,
        lba: u32,
    }

    let mut planned = Vec::with_capacity(files.len());
    for file in files {
        let len = file.content.len();
        let sectors = if len == 0 {
            1
        } else {
            len.div_ceil(SECTOR_SIZE) as u32
        };
        let iso_ident = to_iso_level1_ident(&file.name);
        planned.push(PlannedFile {
            file,
            iso_ident,
            lba: current_lba,
        });
        current_lba += sectors;
    }

    let total_sectors = current_lba;
    let mut image = vec![0u8; (total_sectors as usize) * SECTOR_SIZE];

    // 1. Build Root Directory Sector (LBA 20)
    let mut root_dir = Vec::with_capacity(SECTOR_SIZE);

    // "." entry (self) with Rock Ridge SP + PX
    let dot_entry = build_dir_record(
        &[0x00],
        root_dir_lba,
        SECTOR_SIZE as u32,
        0x02, // Directory flag
        Some(("", true, true)),
    );
    root_dir.extend_from_slice(&dot_entry);

    // ".." entry (parent) with Rock Ridge PX
    let dotdot_entry = build_dir_record(
        &[0x01],
        root_dir_lba,
        SECTOR_SIZE as u32,
        0x02, // Directory flag
        Some(("", false, true)),
    );
    root_dir.extend_from_slice(&dotdot_entry);

    // File entries with Rock Ridge NM (exact filename like "meta-data", "user-data")
    for pf in &planned {
        let rec = build_dir_record(
            pf.iso_ident.as_bytes(),
            pf.lba,
            pf.file.content.len() as u32,
            0x00, // Regular file
            Some((&pf.file.name, false, false)),
        );
        if root_dir.len() + rec.len() > SECTOR_SIZE {
            bail!("Root directory exceeds single sector limit (2048 bytes)");
        }
        root_dir.extend_from_slice(&rec);
    }

    let root_offset = (root_dir_lba as usize) * SECTOR_SIZE;
    image[root_offset..root_offset + root_dir.len()].copy_from_slice(&root_dir);

    // 2. Write L-Path Table (LBA 18) and M-Path Table (LBA 19)
    // Root directory path table record:
    // [0]: len_di = 1
    // [1]: ext_attr_len = 0
    // [2..6]: extent LBA (4 bytes LE or BE)
    // [6..8]: parent dir number (1, 2 bytes LE or BE)
    // [8]: dir identifier (0x00)
    // [9]: padding (0x00)
    let l_path_offset = 18 * SECTOR_SIZE;
    image[l_path_offset] = 1;
    image[l_path_offset + 1] = 0;
    image[l_path_offset + 2..l_path_offset + 6].copy_from_slice(&root_dir_lba.to_le_bytes());
    image[l_path_offset + 6..l_path_offset + 8].copy_from_slice(&1u16.to_le_bytes());
    image[l_path_offset + 8] = 0;
    image[l_path_offset + 9] = 0;

    let m_path_offset = 19 * SECTOR_SIZE;
    image[m_path_offset] = 1;
    image[m_path_offset + 1] = 0;
    image[m_path_offset + 2..m_path_offset + 6].copy_from_slice(&root_dir_lba.to_be_bytes());
    image[m_path_offset + 6..m_path_offset + 8].copy_from_slice(&1u16.to_be_bytes());
    image[m_path_offset + 8] = 0;
    image[m_path_offset + 9] = 0;

    // 3. Build Primary Volume Descriptor (PVD) at LBA 16
    let pvd_offset = 16 * SECTOR_SIZE;
    let pvd = &mut image[pvd_offset..pvd_offset + SECTOR_SIZE];
    pvd[0] = 0x01; // Type 1: Primary Volume Descriptor
    pvd[1..6].copy_from_slice(b"CD001");
    pvd[6] = 0x01; // Version 1
    pvd[7] = 0x00;

    // System Identifier (32 bytes, space-padded)
    write_padded_str(&mut pvd[8..40], "LINUX");
    // Volume Identifier (32 bytes, space-padded) - "cidata" for cloud-init NoCloud!
    write_padded_str(&mut pvd[40..72], volume_label);

    // Volume Space Size (Total sectors, both-endian u32 at 80..88)
    write_both_u32(&mut pvd[80..88], total_sectors);

    // Volume Set Size = 1 (at 120..124)
    write_both_u16(&mut pvd[120..124], 1);
    // Volume Sequence Number = 1 (at 124..128)
    write_both_u16(&mut pvd[124..128], 1);
    // Logical Block Size = 2048 (at 128..132)
    write_both_u16(&mut pvd[128..132], SECTOR_SIZE as u16);

    // Path Table Size = 10 bytes (at 132..140)
    write_both_u32(&mut pvd[132..140], 10);
    // Location of Type-L Path Table (LE u32 at 140..144)
    pvd[140..144].copy_from_slice(&18u32.to_le_bytes());
    // Location of Optional Type-L Path Table (0 at 144..148)
    pvd[144..148].copy_from_slice(&0u32.to_le_bytes());
    // Location of Type-M Path Table (BE u32 at 148..152)
    pvd[148..152].copy_from_slice(&19u32.to_be_bytes());
    // Location of Optional Type-M Path Table (0 at 152..156)
    pvd[152..156].copy_from_slice(&0u32.to_be_bytes());

    // Root Directory Record in PVD (34 bytes at 156..190)
    let pvd_root_record = build_dir_record(&[0x00], root_dir_lba, SECTOR_SIZE as u32, 0x02, None);
    pvd[156..156 + 34].copy_from_slice(&pvd_root_record[..34]);

    // Volume Set / Publisher / Data Preparer / Application Identifiers
    write_padded_str(&mut pvd[190..318], "");
    write_padded_str(&mut pvd[318..446], "ALPASS / MULTIPASS FOR AL2023");
    write_padded_str(&mut pvd[446..574], "CLOUD-INIT NOCLOUD SEED GENERATOR");
    write_padded_str(&mut pvd[574..702], "MP-ISO9660 PURE RUST");
    write_padded_str(&mut pvd[702..813], "");

    // Dates (17 bytes each): "2026112012000000\0"
    let date_bytes = b"2026112012000000\0";
    pvd[813..830].copy_from_slice(date_bytes); // Creation
    pvd[830..847].copy_from_slice(date_bytes); // Modification
    pvd[847..864].copy_from_slice(b"0000000000000000\0"); // Expiration
    pvd[864..881].copy_from_slice(date_bytes); // Effective
    pvd[881] = 0x01; // File Structure Version 1

    // 4. Build Volume Descriptor Set Terminator (VDST) at LBA 17
    let vdst_offset = 17 * SECTOR_SIZE;
    let vdst = &mut image[vdst_offset..vdst_offset + SECTOR_SIZE];
    vdst[0] = 0xFF; // Type 255: Terminator
    vdst[1..6].copy_from_slice(b"CD001");
    vdst[6] = 0x01;

    // 5. Write file contents into their planned sectors
    for pf in &planned {
        let start = (pf.lba as usize) * SECTOR_SIZE;
        let data = &pf.file.content;
        image[start..start + data.len()].copy_from_slice(data);
    }

    Ok(image)
}

/// Convenience helper to write the `cidata` ISO directly to a file path (`seed.iso`).
pub fn write_cidata_iso(path: &Path, volume_label: &str, files: &[IsoFile]) -> Result<()> {
    let bytes = build_cidata_iso(volume_label, files)?;
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn to_iso_level1_ident(name: &str) -> String {
    // Convert e.g. "meta-data" -> "META_DATA.;1" for pure ISO9660 fallback,
    // while Rock Ridge NM carries the exact "meta-data" name.
    let mut s: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    if !s.contains('.') {
        s.push('.');
    }
    s.push_str(";1");
    s
}

/// Builds an ISO9660 directory record with optional Rock Ridge SUSP (`SP`, `PX`, `NM`) entries.
fn build_dir_record(
    ident: &[u8],
    extent_lba: u32,
    data_len: u32,
    flags: u8,
    rock_ridge: Option<(&str, bool, bool)>, // (alt_name, include_sp, is_dir)
) -> Vec<u8> {
    let mut susp = Vec::new();
    if let Some((alt_name, include_sp, is_dir)) = rock_ridge {
        if include_sp {
            // SUSP 'SP' signature entry (7 bytes): 'S', 'P', len=7, ver=1, 0xBE, 0xEF, len_skp=0
            susp.extend_from_slice(&[b'S', b'P', 7, 1, 0xBE, 0xEF, 0]);
        }
        // POSIX file attributes 'PX' entry (36 bytes):
        // 'P', 'X', len=36, ver=1, mode(8), nlinks(8), uid(8), gid(8)
        let mode: u32 = if is_dir {
            0o040755 // directory rwxr-xr-x
        } else {
            0o100644 // regular file rw-r--r--
        };
        let nlinks: u32 = if is_dir { 2 } else { 1 };
        let mut px = [0u8; 36];
        px[0] = b'P';
        px[1] = b'X';
        px[2] = 36;
        px[3] = 1;
        write_both_u32(&mut px[4..12], mode);
        write_both_u32(&mut px[12..20], nlinks);
        write_both_u32(&mut px[20..28], 0); // uid 0
        write_both_u32(&mut px[28..36], 0); // gid 0
        susp.extend_from_slice(&px);

        if !alt_name.is_empty() {
            // Rock Ridge 'NM' Alternate Name entry:
            // 'N', 'M', len = 5 + name_len, ver = 1, flags = 0, name_bytes...
            let name_bytes = alt_name.as_bytes();
            let nm_len = (5 + name_bytes.len()) as u8;
            susp.push(b'N');
            susp.push(b'M');
            susp.push(nm_len);
            susp.push(1);
            susp.push(0); // flags = 0 (complete name in this record)
            susp.extend_from_slice(name_bytes);
        }
        // Ensure SUSP is even length
        if susp.len() % 2 != 0 {
            susp.push(0);
        }
    }

    let base_len = 33 + ident.len();
    let pad_after_ident = if ident.len() % 2 == 0 { 1 } else { 0 };
    let total_len = base_len + pad_after_ident + susp.len();

    let mut rec = vec![0u8; total_len];
    rec[0] = total_len as u8;
    rec[1] = 0; // Extended Attribute Record Length
    write_both_u32(&mut rec[2..10], extent_lba);
    write_both_u32(&mut rec[10..18], data_len);

    // Recording Date & Time (7 bytes): year since 1900, month, day, hour, min, sec, tz
    rec[18..25].copy_from_slice(&[126, 11, 20, 12, 0, 0, 0]);
    rec[25] = flags;
    rec[26] = 0; // File Unit Size
    rec[27] = 0; // Interleave Gap Size
    write_both_u16(&mut rec[28..32], 1); // Volume Sequence Number = 1
    rec[32] = ident.len() as u8;
    rec[33..33 + ident.len()].copy_from_slice(ident);

    let susp_offset = 33 + ident.len() + pad_after_ident;
    rec[susp_offset..susp_offset + susp.len()].copy_from_slice(&susp);

    rec
}

fn write_padded_str(buf: &mut [u8], s: &str) {
    buf.fill(b' ');
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(buf.len());
    buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
}

fn write_both_u16(buf: &mut [u8], val: u16) {
    buf[0..2].copy_from_slice(&val.to_le_bytes());
    buf[2..4].copy_from_slice(&val.to_be_bytes());
}

fn write_both_u32(buf: &mut [u8], val: u32) {
    buf[0..4].copy_from_slice(&val.to_le_bytes());
    buf[4..8].copy_from_slice(&val.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cidata_iso_structure() {
        let files = vec![
            IsoFile::new("meta-data", b"instance-id: test-1\nlocal-hostname: al2023\n".to_vec()),
            IsoFile::new("user-data", b"#cloud-config\nusers:\n  - default\n".to_vec()),
            IsoFile::new("vendor-data", b"#cloud-config\npackage_update: false\n".to_vec()),
        ];

        let iso = build_cidata_iso("cidata", &files).expect("ISO generation should succeed");
        // Verify sector alignment
        assert_eq!(iso.len() % SECTOR_SIZE, 0);
        // Verify PVD magic at sector 16
        let pvd = &iso[16 * SECTOR_SIZE..17 * SECTOR_SIZE];
        assert_eq!(&pvd[1..6], b"CD001");
        // Verify Volume ID is "cidata" padded with spaces
        assert!(pvd[40..72].starts_with(b"cidata                          "));
        // Verify Rock Ridge NM entries exist in Root Directory (sector 20)
        let root_dir = &iso[20 * SECTOR_SIZE..21 * SECTOR_SIZE];
        assert!(root_dir.windows(9).any(|w| w == b"meta-data"));
        assert!(root_dir.windows(9).any(|w| w == b"user-data"));
        assert!(root_dir.windows(11).any(|w| w == b"vendor-data"));
    }
}
