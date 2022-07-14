use crate::{Env, Error};
use dallo::{ModuleId, Ser};
use rkyv::{Archive, Deserialize, Infallible, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use crate::Error::PersistenceError;

#[derive(Default)]
pub struct World {
    environments: BTreeMap<ModuleId, Env>,
    storage_path: PathBuf,
}

impl World {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        World {
            environments: BTreeMap::new(),
            storage_path: path.into(),
        }
    }

    pub fn ephemeral() -> Result<Self, Error> {
        Ok(World {
            environments: BTreeMap::new(),
            storage_path: tempdir().map_err(PersistenceError)?.path().into(),
            // storage_path: PathBuf::from("/tmp/"),
        })
    }

    pub fn deploy(&mut self, env: Env) -> ModuleId {
        let id = env.id();
        self.environments.insert(id, env);
        id
    }

    pub fn query<Arg, Ret>(&self, m_id: ModuleId, name: &str, arg: Arg) -> Result<Ret, Error>
    where
        Arg: for<'a> Serialize<Ser<'a>>,
        Ret: Archive + core::fmt::Debug,
        Ret::Archived: Deserialize<Ret, Infallible> + core::fmt::Debug,
    {
        self.environments
            .get(&m_id)
            .expect("invalid module id")
            .query(name, arg)
    }

    pub fn transact<Arg, Ret>(&mut self, m_id: ModuleId, name: &str, arg: Arg) -> Result<Ret, Error>
    where
        Arg: for<'a> Serialize<Ser<'a>>,
        Ret: Archive + core::fmt::Debug,
        Ret::Archived: Deserialize<Ret, Infallible> + core::fmt::Debug,
    {
        self.environments
            .get_mut(&m_id)
            .expect("invalid module id")
            .transact(name, arg)
    }

    pub fn storage_path(&self) -> &Path {
        self.storage_path.as_path()
    }
}
