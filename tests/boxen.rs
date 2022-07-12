use hatchery::{module, Error, World};

#[ignore]
pub fn box_set_get() -> Result<(), Error> {
    let mut world = World::new();

    let id = world.deploy(module!("box", 1)?);

    let value: Option<i32> = world.query(id, "get", ())?;

    assert_eq!(value, None);

    world.transact(id, "set", 0x11)?;

    let value: Option<i16> = world.query(id, "get", ())?;

    assert_eq!(value, Some(0x11));

    Ok(())
}

#[test]
pub fn box_get() -> Result<(), Error> {
    let mut world = World::new();

    let id = world.deploy(module!("box", 1)?);

    let value: Option<i16> = world.query(id, "get", ())?;

    assert_eq!(value, Some(0x11));

    Ok(())
}
