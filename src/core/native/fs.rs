pub(crate) mod sg {
    use crate::core::{
        native::{
            NativeFunctionContext,
            nativeobject::{NativeObject, sg::FileHandle},
        },
        runtime::{RuntimeError, RuntimeResult, value::RuntimeValue},
    };

    pub fn write(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        context.assert_arguments("sg::write", &["filepath: string", "data: auto T"])?;

        let runtime = context.runtime;
        let arguments = context.arguments;
        let value = &arguments[1];

        let filename = match &arguments[0] {
            RuntimeValue::String(value) => value,
            other => {
                return Err(RuntimeError::AnnotationError {
                    expected: "string".to_string(),
                    found: other.data_type()?.to_string(),
                });
            }
        };

        let bytes = runtime.serialize_into(value)?;

        std::fs::write(filename, bytes)?;

        Ok(RuntimeValue::None)
    }

    pub fn read(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        context.assert_generics("sg::read", &["T"])?;

        let filename = match &context.arguments[0] {
            RuntimeValue::String(value) => value,
            other => {
                return Err(RuntimeError::AnnotationError {
                    expected: "string".to_string(),
                    found: other.data_type()?.to_string(),
                });
            }
        };

        let bytes =
            std::fs::read(filename).unwrap_or_else(|_| panic!("could not find file {filename}"));

        let value = context.runtime.deserialize(&context.generics[0], &bytes)?;

        Ok(value)
    }

    pub fn open_file(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
        use std::{cell::RefCell, rc::Rc};

        context.assert_arguments("sg::open_file", &["filepath: string"])?;

        let filename = match &context.arguments[0] {
            RuntimeValue::String(value) => value,
            other => {
                return Err(RuntimeError::AnnotationError {
                    expected: "string".to_string(),
                    found: other.data_type()?.to_string(),
                });
            }
        };

        let file = std::fs::File::open(filename)?;

        Ok(RuntimeValue::NativeObject(Rc::new(RefCell::new(
            NativeObject::File(Rc::new(RefCell::new(FileHandle::new(
                filename.into(),
                file,
            )))),
        ))))
    }

    pub(crate) mod file_handle {
        use crate::core::{
            native::{
                NativeFunctionContext,
                nativeobject::{NativeObject, sg::FileHandle},
            },
            runtime::{RuntimeError, RuntimeResult, RuntimeValue},
        };

        use std::{cell::RefCell, io::Read, rc::Rc};

        fn get_file_handle(
            context: &NativeFunctionContext,
        ) -> RuntimeResult<Rc<RefCell<FileHandle>>> {
            let object = match &context.arguments[0] {
                RuntimeValue::Reference(reference) => match &*reference.borrow() {
                    RuntimeValue::NativeObject(object) => object.clone(),

                    other => {
                        return Err(RuntimeError::AnnotationError {
                            expected: "sg::FileHandle".to_string(),
                            found: other.data_type()?.to_string(),
                        });
                    }
                },

                RuntimeValue::NativeObject(object) => object.clone(),

                other => {
                    return Err(RuntimeError::AnnotationError {
                        expected: "sg::FileHandle".to_string(),
                        found: other.data_type()?.to_string(),
                    });
                }
            };

            match &*object.borrow() {
                NativeObject::File(handle) => Ok(handle.clone()),
            }
        }

        pub fn read(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            context.assert_generics("sg::FileHandle::read", &["T"])?;

            let handle = get_file_handle(&context)?;

            let mut bytes = Vec::new();

            {
                let handle = handle.borrow();
                let file = handle.file()?;

                file.borrow_mut().read_to_end(&mut bytes)?;
            }

            context.runtime.deserialize(&context.generics[0], &bytes)
        }

        pub fn read_value(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            context.assert_generics("sg::FileHandle::read_value", &["T"])?;

            let data_type = &context.generics[0];
            let size = data_type.static_size(context.runtime)?;

            let mut bytes = vec![0u8; size];

            {
                let handle = get_file_handle(&context)?;
                let file = handle.borrow().file()?;

                file.borrow_mut().read_exact(&mut bytes)?;
            }

            context.runtime.deserialize(data_type, &bytes)
        }

        pub fn rename(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            context.assert_arguments("sg::FileHandle::rename", &["new_path: string"])?;

            let handle = get_file_handle(&context)?;

            let new_path = match &context.arguments[1] {
                RuntimeValue::String(path) => path.clone(),

                other => {
                    return Err(RuntimeError::AnnotationError {
                        expected: "string".to_string(),
                        found: other.data_type()?.to_string(),
                    });
                }
            };

            let mut handle = handle.borrow_mut();

            std::fs::rename(handle.path(), &new_path)?;

            handle.set_path(new_path.into());

            Ok(RuntimeValue::None)
        }

        pub fn close(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            let handle = get_file_handle(&context)?;
            handle.borrow_mut().close();
            Ok(RuntimeValue::None)
        }

        pub fn delete(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            let file = get_file_handle(&context)?;
            file.borrow().delete()?;
            Ok(RuntimeValue::None)
        }
    }
}
