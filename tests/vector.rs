use hatchery::{module, Error, World};

#[ignore]
pub fn vector_push_pop() -> Result<(), Error> {
    let mut world = World::new();

    let id = world.deploy(module!("vector", 6)?);

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
