use hatchery::{module, Error, World};

#[ignore]
pub fn counter_trivial() -> Result<(), Error> {
    let mut world = World::new();

    let id = world.deploy(module!("counter", 3)?);

    let value: i32 = world.query(id, "read_value", ())?;

    assert_eq!(value, 0xfc);

    Ok(())
}

#[ignore]
pub fn counter_increment() -> Result<(), Error> {
    let mut world = World::new();

    let id = world.deploy(module!("counter", 4)?);

    world.transact(id, "increment", ())?;

    let value: i32 = world.query(id, "read_value", ())?;
    assert_eq!(value, 0xfd);

    world.transact(id, "increment", ())?;

    let value: i32 = world.query(id, "read_value", ())?;
    assert_eq!(value, 0xfe);

    Ok(())
}

#[ignore]
pub fn counter_mogrify() -> Result<(), Error> {
    let mut world = World::new();

    let id = world.deploy(module!("counter", 5)?);

    let value: i32 = world.transact(id, "mogrify", 32)?;

    assert_eq!(value, 0xfc);

    let value: i32 = world.query(id, "read_value", ())?;
    assert_eq!(value, 0xfc - 32);

    Ok(())
}
