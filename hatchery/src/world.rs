use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dallo::{ModuleId, MODULE_ID_BYTES};
use parking_lot::RwLock;
use tempfile::tempdir;
use wasmer;

use crate::error::Error;
use crate::session::Session;
use crate::storage_helpers::module_id_to_name;
use crate::store::new_store;

#[derive(Debug, Clone)]
pub struct World {
    modules: Arc<RwLock<BTreeMap<ModuleId, wasmer::Module>>>,
    storage_path: PathBuf,
}

impl World {
    pub fn restore_or_create<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(World {
            modules: Arc::new(RwLock::new(BTreeMap::new())),
            storage_path: path.as_ref().to_owned(),
        })
    }

    pub fn ephemeral() -> Result<Self, Error> {
        let storage_path =
            tempdir().map_err(Error::PersistenceError)?.path().into();

        Ok(World {
            modules: Arc::new(RwLock::new(BTreeMap::new())),
            storage_path,
        })
    }

    pub fn session(&self) -> Session {
        Session::new(self.clone())
    }

    pub fn storage_path(&self) -> &Path {
        self.storage_path.as_path()
    }

    pub fn memory_path(&self, module_id: &ModuleId) -> PathBuf {
        self.storage_path().join(module_id_to_name(*module_id))
    }

    pub fn persist(&self) -> Result<(), Error> {
        todo!()
        // for (module_id, environment) in w.environments.iter() {
        //     let memory_path = MemoryPath::new(self.memory_path(module_id));
        //     let snapshot = Snapshot::new(&memory_path)?;
        //     environment.inner_mut().set_snapshot_id(snapshot.id());
        //     snapshot.save(&memory_path)?;
        // }
        // Ok(())
    }

    pub fn restore(&self) -> Result<(), Error> {
        // let guard = self.0.lock();
        // let w = unsafe { &mut *guard.get() };
        todo!();
        // for (module_id, environment) in w.environments.iter() {
        //     let memory_path = MemoryPath::new(self.memory_path(module_id));
        //     if let Some(snapshot_id) = environment.inner().snapshot_id() {
        //         let snapshot = Snapshot::from_id(*snapshot_id,
        // &memory_path)?;         snapshot.load(&memory_path)?;
        //         println!(
        //             "restored state of module: {:?} from file: {:?}",
        //             module_id_to_name(*module_id),
        //             snapshot.path()
        //         );
        //     }
        // }
        // Ok(())
    }

    pub fn deploy(&mut self, bytecode: &[u8]) -> Result<ModuleId, Error> {
        let id_bytes: [u8; MODULE_ID_BYTES] = blake3::hash(bytecode).into();
        let module_id = ModuleId::from(id_bytes);

        let store = new_store(self.storage_path());

        let module = wasmer::Module::new(&store, bytecode)?;

        self.modules.write().insert(module_id, module);

        Ok(module_id)
    }

    pub fn get_module(&self, module_id: ModuleId) -> wasmer::Module {
        self.modules
            .read()
            .get(&module_id)
            .expect("Invalid module")
            .clone()
    }
}
