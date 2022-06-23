use hatchery::{module, ModuleError};
use rend::LittleEndian;

#[test]
pub fn counter_trivial() -> Result<(), ModuleError> {
    let mut module = module!("counter")?;
    let value: LittleEndian<i32> = module.call("read_value", ())?;

    assert_eq!(value, 0);

    Ok(())
}

#[test]
pub fn counter_increment() -> Result<(), ModuleError> {
    let mut module = module!("counter")?;

    module.call("mogrify", 0xee)?;

    let value: LittleEndian<i32> = module.call("read_value", ())?;

    assert_eq!(value, 1);

    Ok(())
}
