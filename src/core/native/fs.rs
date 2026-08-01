pub(crate) mod sg {
    use crate::core::{
        native::{
            NativeFunctionContext,
            fs::sg,
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
            NativeObject::File(sg::FileHandle::new(file)),
        ))))
    }

    pub(crate) mod native_file {
        use crate::core::{
            native::{NativeFunctionContext, nativeobject::NativeObject},
            runtime::{RuntimeError, RuntimeResult, RuntimeValue},
        };
        use std::io::Read;

        pub fn read(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            context.assert_generics("sg::NativeFile::read", &["T"])?;

            let file = match &context.arguments[0] {
                RuntimeValue::Reference(reference) => match &*reference.borrow() {
                    RuntimeValue::NativeObject(object) => object.clone(),

                    other => {
                        return Err(RuntimeError::AnnotationError {
                            expected: "NativeFile".to_string(),
                            found: other.data_type()?.to_string(),
                        });
                    }
                },

                RuntimeValue::NativeObject(object) => object.clone(),

                other => {
                    return Err(RuntimeError::AnnotationError {
                        expected: "NativeFile".to_string(),
                        found: other.data_type()?.to_string(),
                    });
                }
            };

            let mut object = file.borrow_mut();

            let NativeObject::File(native_file) = &mut *object;

            let mut bytes = Vec::new();

            native_file.file().borrow_mut().read_to_end(&mut bytes)?;

            context.runtime.deserialize(&context.generics[0], &bytes)
        }

        pub fn read_value(context: NativeFunctionContext) -> RuntimeResult<RuntimeValue> {
            context.assert_generics("sg::NativeFile::read_value", &["T"])?;

            let file = match &context.arguments[0] {
                RuntimeValue::Reference(reference) => match &*reference.borrow() {
                    RuntimeValue::NativeObject(object) => object.clone(),

                    other => {
                        return Err(RuntimeError::AnnotationError {
                            expected: "NativeFile".to_string(),
                            found: other.data_type()?.to_string(),
                        });
                    }
                },

                RuntimeValue::NativeObject(object) => object.clone(),

                other => {
                    return Err(RuntimeError::AnnotationError {
                        expected: "NativeFile".to_string(),
                        found: other.data_type()?.to_string(),
                    });
                }
            };

            let data_type = &context.generics[0];

            let size = data_type.static_size(context.runtime)?;

            let mut bytes = vec![0u8; size];

            {
                let mut object = file.borrow_mut();

                let NativeObject::File(native_file) = &mut *object;

                native_file.file().borrow_mut().read_exact(&mut bytes)?;
            }

            context.runtime.deserialize(data_type, &bytes)
        }
    }
}
