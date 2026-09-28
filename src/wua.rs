//! Cemu's compressed Wii U archive (.wua), which is Exzap's ZArchive format
//! (MIT-0): the files' bytes one after another, cut into 64 KiB blocks that
//! are each stored as zstd, or as they are when that saved nothing, then a
//! table of where the blocks start, the names, the file tree, and a footer
//! that says where those are. Every number is big-endian. Cemu writes the
//! files already decrypted, so reading them needs no key.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const MAGIC: u32 = 0x169f_52d6;
const VERSION: u32 = 0x61bf_3a01;
const FOOTER_SIZE: u64 = 144;
const BLOCK: usize = 64 * 1024;
/// Each offset record gives where one block starts and the stored sizes of
/// it and the fifteen after it, less one.
const BLOCKS_PER_RECORD: usize = 16;
const RECORD_SIZE: usize = 8 + 2 * BLOCKS_PER_RECORD;
/// A node of the file tree: a name and a flag, then a file's offset and size
/// or a folder's first child and count.
const NODE_SIZE: usize = 16;
const IS_FILE: u32 = 0x8000_0000;

pub struct Archive {
    file: File,
    data_start: u64,
    records: Vec<u8>,
    names: Vec<u8>,
    tree: Vec<u8>,
}

/// A file in the archive, and where its bytes sit once the blocks are
/// unpacked end to end.
pub struct Entry {
    pub path: String,
    offset: u64,
    size: u64,
}

fn be16(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([bytes[at], bytes[at + 1]])
}

fn be32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn be64(bytes: &[u8], at: usize) -> u64 {
    u64::from_be_bytes(bytes[at..at + 8].try_into().unwrap())
}

fn read_at(file: &mut File, offset: u64, length: usize) -> std::io::Result<Vec<u8>> {
    let mut bytes = vec![0; length];
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn damaged() -> String {
    "The .wua file is damaged. Have Cemu make it again.".to_string()
}

impl Archive {
    pub fn open(path: &Path) -> Result<Self, String> {
        let not_wua = || "That isn't a .wua file made by Cemu.".to_string();
        let mut file = File::open(path).map_err(|_| "Couldn't open the game's .wua file.".to_string())?;
        let size = file.metadata().map_err(|_| not_wua())?.len();
        if size < FOOTER_SIZE {
            return Err(not_wua());
        }
        let footer = read_at(&mut file, size - FOOTER_SIZE, FOOTER_SIZE as usize).map_err(|_| not_wua())?;
        if be32(&footer, 140) != MAGIC || be32(&footer, 136) != VERSION || be64(&footer, 128) != size {
            return Err(not_wua());
        }
        let section = |index: usize| (be64(&footer, index * 16), be64(&footer, index * 16 + 8));
        let mut load = |(offset, length): (u64, u64)| {
            if offset.checked_add(length).is_none_or(|end| end > size) {
                return Err(damaged());
            }
            read_at(&mut file, offset, length as usize).map_err(|_| damaged())
        };
        let records = load(section(1))?;
        let names = load(section(2))?;
        let tree = load(section(3))?;
        Ok(Self {
            file,
            data_start: section(0).0,
            records,
            names,
            tree,
        })
    }

    /// A name from the name table: one byte of length, or two when the
    /// first has its top bit set, then the text.
    fn name(&self, offset: usize) -> String {
        let Some(&first) = self.names.get(offset) else {
            return String::new();
        };
        let (length, start) = if first & 0x80 == 0 {
            (usize::from(first), offset + 1)
        } else {
            let second = self.names.get(offset + 1).copied().unwrap_or(0);
            (usize::from(first & 0x7f) | usize::from(second) << 7, offset + 2)
        };
        self.names
            .get(start..start + length)
            .map(|text| String::from_utf8_lossy(text).into_owned())
            .unwrap_or_default()
    }

    /// Every file in the archive, with its path from the root.
    pub fn files(&self) -> Vec<Entry> {
        let mut found = Vec::new();
        self.walk(0, "", &mut found, 0);
        found
    }

    fn walk(&self, folder: usize, prefix: &str, found: &mut Vec<Entry>, depth: usize) {
        // A damaged tree could point back at itself; real games are shallow.
        let Some(node) = self.tree.get(folder * NODE_SIZE..(folder + 1) * NODE_SIZE) else {
            return;
        };
        if depth > 32 {
            return;
        }
        let (first, count) = (be32(node, 4) as usize, be32(node, 8) as usize);
        for child in first..first.saturating_add(count) {
            let Some(node) = self.tree.get(child * NODE_SIZE..(child + 1) * NODE_SIZE) else {
                return;
            };
            let head = be32(node, 0);
            let path = format!("{prefix}{}", self.name((head & !IS_FILE) as usize));
            if head & IS_FILE != 0 {
                let high = u64::from(be32(node, 12));
                found.push(Entry {
                    path,
                    offset: u64::from(be32(node, 4)) | (high & 0xffff) << 32,
                    size: u64::from(be32(node, 8)) | (high & 0xffff_0000) << 16,
                });
            } else {
                self.walk(child, &format!("{path}/"), found, depth + 1);
            }
        }
    }

    /// One block, unpacked.
    fn block(&mut self, index: usize) -> Result<Vec<u8>, String> {
        let at = index / BLOCKS_PER_RECORD * RECORD_SIZE;
        let record = self.records.get(at..at + RECORD_SIZE).ok_or_else(damaged)?;
        let sub = index % BLOCKS_PER_RECORD;
        let offset = be64(record, 0) + (0..sub).map(|k| u64::from(be16(record, 8 + 2 * k)) + 1).sum::<u64>();
        let stored = usize::from(be16(record, 8 + 2 * sub)) + 1;
        let bytes = read_at(&mut self.file, self.data_start + offset, stored).map_err(|_| damaged())?;
        if stored == BLOCK {
            return Ok(bytes);
        }
        zstd::bulk::decompress(&bytes, BLOCK).map_err(|_| damaged())
    }

    pub fn read(&mut self, entry: &Entry) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::with_capacity(entry.size as usize);
        let end = entry.offset + entry.size;
        let mut at = entry.offset;
        while at < end {
            let block = self.block((at / BLOCK as u64) as usize)?;
            let from = (at % BLOCK as u64) as usize;
            if from >= block.len() {
                return Err(damaged());
            }
            let take = (block.len() - from).min((end - at) as usize);
            bytes.extend_from_slice(&block[from..from + take]);
            at += take as u64;
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small archive the way Cemu lays one out: `content/a.bin` holding
    /// `data`, each block compressed unless that saves nothing, when it is
    /// stored as it is.
    fn build(data: &[u8]) -> Vec<u8> {
        let blocks: Vec<Vec<u8>> = data
            .chunks(BLOCK)
            .map(|chunk| {
                let packed = zstd::bulk::compress(chunk, 3).unwrap();
                if packed.len() >= BLOCK && chunk.len() == BLOCK {
                    chunk.to_vec()
                } else {
                    packed
                }
            })
            .collect();
        let mut out: Vec<u8> = blocks.concat();
        let data_size = out.len() as u64;

        let records_at = out.len() as u64;
        let mut before = 0u64;
        for group in blocks.chunks(BLOCKS_PER_RECORD) {
            out.extend_from_slice(&before.to_be_bytes());
            for k in 0..BLOCKS_PER_RECORD {
                let size = group.get(k).map_or(0, |b| (b.len() - 1) as u16);
                out.extend_from_slice(&size.to_be_bytes());
            }
            before += group.iter().map(|b| b.len() as u64).sum::<u64>();
        }
        let records_size = out.len() as u64 - records_at;

        let names_at = out.len() as u64;
        out.extend_from_slice(&[0, 7]);
        out.extend_from_slice(b"content");
        out.push(5);
        out.extend_from_slice(b"a.bin");
        let names_size = out.len() as u64 - names_at;

        let tree_at = out.len() as u64;
        let node = |out: &mut Vec<u8>, words: [u32; 4]| words.iter().for_each(|w| out.extend_from_slice(&w.to_be_bytes()));
        node(&mut out, [0, 1, 1, 0]);
        node(&mut out, [1, 2, 1, 0]);
        node(&mut out, [IS_FILE | 9, 0, data.len() as u32, 0]);
        let tree_size = out.len() as u64 - tree_at;

        let total = out.len() as u64 + FOOTER_SIZE;
        for (offset, size) in [(0, data_size), (records_at, records_size), (names_at, names_size), (tree_at, tree_size), (0, 0), (0, 0)] {
            out.extend_from_slice(&u64::to_be_bytes(offset));
            out.extend_from_slice(&u64::to_be_bytes(size));
        }
        out.extend_from_slice(&[0; 32]);
        out.extend_from_slice(&total.to_be_bytes());
        out.extend_from_slice(&VERSION.to_be_bytes());
        out.extend_from_slice(&MAGIC.to_be_bytes());
        out
    }

    fn written(bytes: &[u8], name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("omoio-portraits-{}-{name}.wua", std::process::id()));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn a_file_reads_back_across_blocks() {
        // A first block that doesn't compress, so it is stored as it is, then
        // a short one that does.
        let mut seed = 0x2545_f491_u32;
        let mut data: Vec<u8> = (0..BLOCK)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed as u8
            })
            .collect();
        data.extend((0..5000).map(|i| (i % 251) as u8));
        let path = written(&build(&data), "blocks");
        let mut archive = Archive::open(&path).unwrap();
        let files = archive.files();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "content/a.bin");
        assert_eq!(archive.read(&files[0]).unwrap(), data);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn anything_else_is_refused() {
        let path = written(&[0; 400], "junk");
        assert!(Archive::open(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
