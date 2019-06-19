quick_error! {
    #[derive(Debug)]
    pub enum UnpackError {
        Variant1 {
            description("no file name specified")
        }
        InvalidPE {
            description("The PE file was invalid!")
            from(pelite::Error)
        }
        IOError {
            from(std::io::Error)
        }
        InvalidWStr {
            description("The wide string was invalid! (This should *never* happen)")
        }
        ResourceNotFound {
            from(pelite::resources::FindError)
        }
        ZlibError {
            description("Couldn't inflate target!")
        }
        InvalidArchive {
            description("The CE Archive was corrupted!")
        }
        MalformedArchive {
            from(std::array::TryFromSliceError)
            from(std::str::Utf8Error)
        }
        BadMagic {
            description("Invalid Trainer Magic (\"CHEAT\"). Trainer too new / too old?")
        }
        BadFilename {
            description("Invalid output filename!")
        }
    }
}
