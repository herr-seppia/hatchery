#[derive(Debug)]
pub enum ModuleError {
    InstantiationError(wasmer::InstantiationError),
    CompileError(wasmer::CompileError),
    ExportError(wasmer::ExportError),
    RuntimeError(wasmer::RuntimeError),
    MissingArgRetBuffer,
}

impl From<wasmer::InstantiationError> for ModuleError {
    fn from(e: wasmer::InstantiationError) -> Self {
        ModuleError::InstantiationError(e)
    }
}

impl From<wasmer::CompileError> for ModuleError {
    fn from(e: wasmer::CompileError) -> Self {
        ModuleError::CompileError(e)
    }
}

impl From<wasmer::ExportError> for ModuleError {
    fn from(e: wasmer::ExportError) -> Self {
        ModuleError::ExportError(e)
    }
}

impl From<wasmer::RuntimeError> for ModuleError {
    fn from(e: wasmer::RuntimeError) -> Self {
        ModuleError::RuntimeError(e)
    }
}
