use hatchery::{contract_bytes, Error, World};
use std::path::PathBuf;
use dallo::ModuleId;

#[test]
pub fn box_set_get() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(hatchery::Env::new(
        contract_bytes!("box"),
        world.storage_path(),
    )?);

    let value: Option<i32> = world.query(id, "get", ())?;

    assert_eq!(value, None);

    world.transact(id, "set", 0x11)?;

    let value: Option<i16> = world.query(id, "get", ())?;

    assert_eq!(value, Some(0x11));

    Ok(())
}

#[test]
pub fn box_set_store_restore_get() -> Result<(), Error> {
    let mut storage_path = PathBuf::new();
    let first_id: ModuleId;

    {
        let mut first_world = World::ephemeral()?;

        first_id = first_world.deploy(hatchery::Env::new(
            contract_bytes!("box"),
            first_world.storage_path(),
        )?);

        first_world.transact(first_id, "set", 0x23)?;

        first_world.storage_path().clone_into(&mut storage_path);
    }

    let mut second_world = World::new(storage_path);

    let second_id = second_world.deploy(hatchery::Env::new(
        contract_bytes!("box"),
        second_world.storage_path(),
    )?);

    assert_eq!(first_id, second_id);

    let value: Option<i16> = second_world.query(second_id, "get", ())?;

    assert_eq!(value, Some(0x23));

    Ok(())
}
