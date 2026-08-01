use std::{cell::RefCell, rc::Rc};

use crate::core::lang::ast::DataType;

#[derive(Debug, PartialEq, Clone)]
pub enum NativeObject {
    File(Rc<RefCell<sg::FileHandle>>),
}

pub type RuntimeNativeObject = Rc<RefCell<NativeObject>>;

impl NativeObject {
    fn data_type_name(&self) -> &'static str {
        match self {
            Self::File(_) => "sg::FileHandle",
        }
    }

    pub fn data_type(&self) -> DataType {
        DataType::NativeObject(self.data_type_name().to_string())
    }
}

/* File */

pub mod sg {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use crate::core::runtime::{RuntimeError, RuntimeResult};

    #[derive(Debug, Clone)]
    pub struct FileHandle {
        file: Option<Rc<RefCell<std::fs::File>>>,
        path: PathBuf,
    }

    impl PartialEq for FileHandle {
        fn eq(&self, other: &Self) -> bool {
            match (&self.file, &other.file) {
                (Some(ours), Some(theirs)) => Rc::ptr_eq(ours, theirs),

                // closed handles cannot share a File anymore, so compare paths
                (None, None) => self.path == other.path,

                _ => false,
            }
        }
    }

    impl FileHandle {
        pub fn new(path: PathBuf, file: std::fs::File) -> Self {
            Self {
                file: Some(Rc::new(RefCell::new(file))),
                path,
            }
        }

        pub fn close(&mut self) {
            self.file = None;
        }

        pub fn file(&self) -> RuntimeResult<Rc<RefCell<std::fs::File>>> {
            self.file
                .clone()
                .ok_or(RuntimeError::NativeError("file is closed"))
        }

        pub fn rename(&mut self, path: PathBuf) -> RuntimeResult<()> {
            if self.file.is_some() {
                return Err(RuntimeError::NativeError("cannot rename an open file"));
            }

            std::fs::rename(&self.path, &path)?;

            self.path = path;

            Ok(())
        }

        pub fn delete(&self) -> RuntimeResult<()> {
            std::fs::remove_file(&self.path)?;
            Ok(())
        }

        pub fn path(&self) -> &PathBuf {
            &self.path
        }

        pub fn set_path(&mut self, path: PathBuf) {
            self.path = path;
        }
    }
}
