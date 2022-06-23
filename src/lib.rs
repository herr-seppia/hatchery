use rkyv::{
    archived_root,
    ser::{serializers::BufferSerializer, Serializer},
    Archive, Deserialize, Infallible, Serialize,
};
use wasmer::{imports, NativeFunc, Val};

mod error;

pub use error::ModuleError;

pub struct Module {
    #[allow(unused)]
    instance: wasmer::Instance,
    arg_ret_ofs: i32,
}

impl Module {
    pub fn new(bytecode: &[u8]) -> Result<Self, ModuleError> {
        let import_object = imports! {};

        let store = wasmer::Store::default();

        let module = wasmer::Module::new(&store, bytecode)?;

        let instance = wasmer::Instance::new(&module, &import_object)?;

        if let Val::I32(arg_ret_ofs) = instance.exports.get_global("AR")?.get() {
            Ok(Module {
                instance,
                arg_ret_ofs,
            })
        } else {
            Err(ModuleError::MissingArgRetBuffer)
        }
    }

    pub fn call<Arg, Ret>(&mut self, name: &str, arg: Arg) -> Result<Ret, ModuleError>
    where
        Arg: for<'a> Serialize<BufferSerializer<&'a mut [u8]>>,
        Ret: Archive,
        Ret::Archived: Deserialize<Ret, Infallible>,
    {
        let fun: NativeFunc<i32, i32> = self.instance.exports.get_native_function(name)?;

        // copy the argument bytes to the arg/ret buffer of the module.
        let mem = self.instance.exports.get_memory("memory")?;

        let ret_entry = {
            let arg_ret = unsafe { &mut mem.data_unchecked_mut()[self.arg_ret_ofs as usize..] };

            let slice = &arg_ret[..128];

            println!("arg_ret:\n{}", pretty_hex::pretty_hex(&slice));

            let mut serializer = BufferSerializer::new(arg_ret);

            let entry = serializer.serialize_value(&arg).unwrap() as i32;

            fun.call(entry)?
        };

        let arg_ret = unsafe { &mut mem.data_unchecked_mut()[self.arg_ret_ofs as usize..] };

        let read_from = &arg_ret[ret_entry as usize..];
        let ret = unsafe { archived_root::<Ret>(read_from) };
        let de = ret.deserialize(&mut Infallible).unwrap();
        Ok(de)
    }
}

#[macro_export]
macro_rules! module {
    ($name:literal) => {
        hatchery::Module::new(include_bytes!(concat!(
            "../target/wasm32-unknown-unknown/release/",
            $name,
            ".wasm"
        )))
    };
}
