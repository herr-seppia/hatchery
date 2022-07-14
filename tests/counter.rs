use hatchery::{contract_bytes, Error, World};

#[test]
pub fn counter_trivial() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(hatchery::Env::new(
        contract_bytes!("counter"),
        world.storage_path(),
    )?);

    let value: i32 = world.query(id, "read_value", ())?;

    assert_eq!(value, 0xfc);

    Ok(())
}

#[test]
pub fn counter_increment() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(hatchery::Env::new(
        contract_bytes!("counter"),
        world.storage_path(),
    )?);

    world.transact(id, "increment", ())?;

    let value: i32 = world.query(id, "read_value", ())?;
    assert_eq!(value, 0xfd);

    world.transact(id, "increment", ())?;

    let value: i32 = world.query(id, "read_value", ())?;
    assert_eq!(value, 0xfe);

    Ok(())
}

#[test]
pub fn counter_mogrify() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(hatchery::Env::new(
        contract_bytes!("counter"),
        world.storage_path(),
    )?);

    let value: i32 = world.transact(id, "mogrify", 32)?;

    assert_eq!(value, 0xfc);

    let value: i32 = world.query(id, "read_value", ())?;
    assert_eq!(value, 0xfc - 32);

    Ok(())
}
