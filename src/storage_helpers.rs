use dallo::ModuleId;

pub fn module_id_to_file_name(module_id: ModuleId) -> String {
    format!("{}", ModuleIdWrapper(module_id))
}

#[macro_export]
macro_rules! contract_bytes {
    ($contract_name:literal) => {
        include_bytes!(concat!(
            "../target/wasm32-unknown-unknown/release/",
            $contract_name,
            ".wasm"
        ))
    };
}

struct ModuleIdWrapper(pub ModuleId);

impl core::fmt::UpperHex for ModuleIdWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bytes = &self.0[..];
        if f.alternate() {
            write!(f, "0x")?
        }
        for byte in bytes {
            write!(f, "{:02X}", &byte)?
        }
        Ok(())
    }
}

impl core::fmt::Display for ModuleIdWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(self, f)
    }
}
