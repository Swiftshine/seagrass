use crate::core::runtime::{Runtime, RuntimeError, RuntimeResult};

#[derive(Debug)]
pub struct SerializationTarget {
    pub data: Vec<u8>,
}

impl SerializationTarget {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl Runtime {
    pub fn create_serialization_target(&mut self) {
        self.destroy_serialization_target();
        self.serialization_target = Some(SerializationTarget::new())
    }

    pub fn destroy_serialization_target(&mut self) {
        self.serialization_target = None;
    }

    pub fn has_serialization_target(&self) -> bool {
        self.serialization_target.is_some()
    }

    pub fn get_serialization_target_mut(&mut self) -> RuntimeResult<&mut SerializationTarget> {
        self.serialization_target
            .as_mut()
            .ok_or(RuntimeError::NoSerializationTarget)
    }
}
