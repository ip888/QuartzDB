use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct WriteAheadLog {
    file: File,
    _path: String, // Keep for future use but prefix with _ to silence warnings
}

impl WriteAheadLog {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            file,
            _path: path.as_ref().to_string_lossy().into_owned(),
        })
    }

    pub fn append(&mut self, record: &[u8]) -> io::Result<u64> {
        let position = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(record)?;
        self.file.flush()?;
        Ok(position)
    }

    pub fn read(&mut self, position: u64, length: usize) -> io::Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(position))?;
        let mut buffer = vec![0; length];
        self.file.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}
