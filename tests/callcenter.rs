use hatchery::{contract_bytes, Error, World};

#[test]
pub fn world_call_counter() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let c_id = world.deploy(hatchery::Env::new(
        contract_bytes!("counter"),
        world.storage_path(),
    )?);

    let value: i32 = world.query(c_id, "read_value", ())?;

    assert_eq!(value, 0xfc);

    Ok(())
}
