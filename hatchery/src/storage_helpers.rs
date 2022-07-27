use dallo::{ModuleId, SnapshotId};

pub fn merge_module_snapshot_names(
    module_name: impl AsRef<str>,
    snapshot_name: impl AsRef<str>,
) -> String {
    format!("{}_{}", module_name.as_ref(), snapshot_name.as_ref())
}

pub fn module_id_to_filename(module_id: ModuleId) -> String {
    format!("{}", ModuleIdWrapper(module_id))
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

pub fn snapshot_id_to_filename(snapshot_id: SnapshotId) -> String {
    format!("{}", SnapshotIdWrapper(snapshot_id))
}

pub fn into_snapshot_id(s: impl AsRef<str>) -> SnapshotId {
    blake3::hash(s.as_ref().as_bytes()).into()
}

struct SnapshotIdWrapper(pub SnapshotId);

impl core::fmt::UpperHex for SnapshotIdWrapper {
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

impl core::fmt::Display for SnapshotIdWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(self, f)
    }
}
