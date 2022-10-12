// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::sync::Mutex;
use wasmer::wasmparser::Operator;
use wasmer::{
    AsStoreMut, ExportIndex, FunctionMiddleware, GlobalInit, GlobalType,
    Instance, LocalFunctionIndex, MiddlewareError, MiddlewareReaderState,
    ModuleMiddleware, Mutability, Type,
};
use wasmer_middlewares::metering::set_remaining_points;
use wasmer_types::{GlobalIndex, ModuleInfo};

#[derive(Debug, Clone)]
struct Indexes {
    remaining_points_global: GlobalIndex,

    limit_global: GlobalIndex,
    spent_global: GlobalIndex,
}

impl Indexes {
    /// The global index for remaining points.
    fn remaining_points(&self) -> GlobalIndex {
        self.remaining_points_global
    }

    /// The global index for points limit.
    fn limit_points(&self) -> GlobalIndex {
        self.limit_global
    }

    /// The global index for points limit.
    fn spent_points(&self) -> GlobalIndex {
        self.spent_global
    }
}

/// The module-level middleware providing limit and spent calls.
///
/// # Panic
///
/// Attempting to use it without a `Metering` middleware will result in a panic.
///
/// Similarly an to an instance of `Metering`, an instance of `LimitSpend`
/// should _not_ be shared among among different modules. Attempts to use a
/// `LimitSpend` instance from multiple modules will result in a panic.
#[derive(Debug)]
pub struct LimitSpent {
    initial_limit: u64,
    /// The global indexes for providing points to the module.
    indexes: Mutex<Option<Indexes>>,
}

impl LimitSpent {
    pub fn new(initial_limit: u64) -> Self {
        Self {
            initial_limit,
            indexes: Mutex::new(None),
        }
    }
}

impl ModuleMiddleware for LimitSpent {
    fn generate_function_middleware(
        &self,
        _: LocalFunctionIndex,
    ) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionLimitSpent {
            indexes: self.indexes.lock().unwrap().clone().unwrap(),
        })
    }

    fn transform_module_info(&self, module_info: &mut ModuleInfo) {
        let mut indexes = self.indexes.lock().unwrap();

        if indexes.is_some() {
            panic!("LimitSpend::transform_module_info: Attempting to use a `LimitSpend` middleware from multiple modules.");
        }

        let remaining_points_global = match module_info
            .exports
            .get("wasmer_metering_remaining_points")
            .expect("LimitSpend::transform_module_info: Couldn't detect `Metering` exports.") {
            ExportIndex::Global(index) => *index,
            _ => unreachable!(),
        };

        // let limit_global = match
        // module_info.exports.get("LIMIT").expect("
        // LimitSpend::transform_module_info: Couldn't detect `LIMIT` export.")
        // {     ExportIndex::Global(index) => *index,
        //     _ => unreachable!(),
        // };
        //
        // let spent_global = match
        // module_info.exports.get("SPENT").expect("
        // LimitSpend::transform_module_info: Couldn't detect `SPENT` export.")
        // {     ExportIndex::Global(index) => *index,
        //     _ => unreachable!(),
        // };

        // Append a global for point `limit` and initialize it.
        let limit_global = module_info
            .globals
            .push(GlobalType::new(Type::I64, Mutability::Var));

        module_info
            .global_initializers
            .push(GlobalInit::I64Const(self.initial_limit as i64));

        module_info
            .exports
            .insert("LIMIT".to_string(), ExportIndex::Global(limit_global));

        // Append a global for points `spent` and initialize it.
        let spent_global = module_info
            .globals
            .push(GlobalType::new(Type::I64, Mutability::Var));

        module_info
            .global_initializers
            .push(GlobalInit::I64Const(self.initial_limit as i64));

        module_info
            .exports
            .insert("SPENT".to_string(), ExportIndex::Global(spent_global));

        *indexes = Some(Indexes {
            remaining_points_global,
            limit_global,
            spent_global,
        });
    }
}

#[derive(Debug)]
pub struct FunctionLimitSpent {
    /// The global indexes for providing points to the module.
    indexes: Indexes,
}

impl FunctionMiddleware for FunctionLimitSpent {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        // if the `spent` global is being accessed, place the correct value, as
        // reported by the `Metering` middleware.
        if let Operator::GlobalGet { global_index } = operator {
            let spent_global = self.indexes.spent_points().as_u32();
            if spent_global == global_index {
                state.extend(&[
                    // l <- limit
                    Operator::GlobalGet {
                        global_index: self.indexes.limit_points().as_u32(),
                    },
                    // r <- remaining
                    Operator::GlobalGet {
                        global_index: self.indexes.remaining_points().as_u32(),
                    },
                    // s = l - r
                    Operator::I64Sub,
                    // spent <- s
                    Operator::GlobalSet {
                        global_index: spent_global,
                    },
                ]);
            }
        }

        state.push_operator(operator);
        Ok(())
    }
}

/// Set the new limit and remaining points in an instance.
///
/// Note: This can be used in a headless engine after an ahead-of-time
/// compilation as all required state lives in the instance.
///
/// # Panic
///
/// The given instance must have been processed with the [`Metering`] and
/// [`LimitSpent`] middlewares at compile time, otherwise this will panic.
/// ```
pub fn set_limit_points(
    ctx: &mut impl AsStoreMut,
    instance: &Instance,
    points: u64,
) {
    set_remaining_points(ctx, instance, points);

    instance
        .exports
        .get_global("LIMIT")
        .expect("Can't get `LIMIT` from Instance")
        .set(ctx, points.into())
        .expect("Can't set `LIMIT` in Instance");
}
