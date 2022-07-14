use hatchery::{contract_bytes, Error, World};

#[test]
pub fn vector_push_pop() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(hatchery::Env::new(
        contract_bytes!("vector"),
        world.storage_path(),
    )?);

    const N: usize = 128;

    for i in 0..N {
        world.transact(id, "push", i)?;
    }

    for i in 0..N {
        let popped: Option<i16> = world.transact(id, "pop", ())?;

        assert_eq!(popped, Some((N - i - 1) as i16));
    }

    let popped: Option<i16> = world.transact(id, "pop", ())?;

    assert_eq!(popped, None);

    Ok(())
}
