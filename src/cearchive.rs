use inflate::inflate_bytes;
use std::convert::TryInto;
use std::str;

use crate::errors::UnpackError;

#[derive(Debug, Clone)]
pub struct CEArchiveFile  {
    pub name: String,
    pub folder: String,
    pub data: Vec<u8>
}

#[derive(Debug, Clone)]
pub struct CEArchive {
    pub filecount: u32,
    pub files: Vec<CEArchiveFile>
}

impl CEArchiveFile {
    pub fn new(name: &str, folder: &str, data: &[u8]) -> CEArchiveFile {
        CEArchiveFile{
            name: name.to_string(),
            folder: folder.to_string(),
            data: data.to_vec()
        }
    }
}

impl CEArchive{
    pub fn new(data: &[u8]) -> Result<CEArchive, UnpackError> {
        let filecount = u32::from_ne_bytes(data[0..4].try_into()?);
        let inflated = match inflate_bytes(&data[4..]) {
            Ok(data) => data,
            _ => {
                return Err(UnpackError::ZlibError);
            }
        };

        let mut offset = 0usize;
        let mut files: Vec<CEArchiveFile> = Vec::new();

        // TODO: Use nom for this
        for _ in 0..filecount {
            let namesize = u32::from_ne_bytes(inflated[offset..offset+4].try_into()?) as usize;
            offset += 4;
            let name = str::from_utf8(&inflated[offset..offset + namesize])?;
            offset += namesize;

            let foldersize = u32::from_ne_bytes(inflated[offset..offset+4].try_into()?) as usize;
            offset += 4;
            let folder = str::from_utf8(&inflated[offset..offset + foldersize])?;
            offset += foldersize;

            let datasize = u32::from_ne_bytes(inflated[offset..offset+4].try_into()?) as usize;
            offset += 4;

            let data = &inflated[offset..offset + datasize];
            offset += datasize;

            files.push(CEArchiveFile::new(name, folder, data));
        }


        Ok(CEArchive{
            filecount,
            files
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ARCHIVE: &[u8] = include_bytes!("/home/jan/ARCHIVE");

    #[test]
    fn filecount() {
        let archive = CEArchive::new(&TEST_ARCHIVE).unwrap();
        assert_eq!(archive.filecount, 5)
    }

    #[test]
    fn contains() {
        let archive = CEArchive::new(&TEST_ARCHIVE).unwrap();
        assert!(archive.files.iter().find(|f| "CET_TRAINER.CETRAINER" == f.name.as_str()).is_some());
    }
}
