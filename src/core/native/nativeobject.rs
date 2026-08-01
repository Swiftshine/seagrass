use std::{cell::RefCell, rc::Rc};

use crate::core::lang::ast::DataType;

#[derive(Debug, PartialEq, Clone)]
pub enum NativeObject {
    File(sg::FileHandle),
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
    use std::{cell::RefCell, rc::Rc};

    #[derive(Debug, Clone)]
    pub struct FileHandle {
        file: Rc<RefCell<std::fs::File>>,
    }

    impl PartialEq for FileHandle {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.file, &other.file)
        }
    }

    impl FileHandle {
        pub fn new(file: std::fs::File) -> Self {
            Self {
                file: Rc::new(RefCell::new(file)),
            }
        }

        pub fn file(&self) -> Rc<RefCell<std::fs::File>> {
            Rc::clone(&self.file)
        }
    }
}
