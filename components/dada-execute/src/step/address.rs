use dada_ir::{code::bir, error, parameter::Parameter};

use crate::{
    error::DiagnosticBuilderExt,
    machine::{Object, ObjectData, Value},
};

use super::Stepper;

/// Identifies a place in memory
#[derive(Copy, Clone, Debug)]
pub(super) enum Address {
    /// A local variable in the top-most stack frame
    Local(bir::LocalVariable),

    /// A constant, like a Class or a Function
    Constant(Value),

    /// A field with the given index of the given object.
    /// If this is a field of a user-declared class (as opposed,
    /// say, to a tuple), then includes the [`Parameter`]
    /// representing that field.
    Field(Object, usize, Option<Parameter>),
}

impl Stepper<'_> {
    /// Read the value at `address`; does not account for permissions at all.
    pub(super) fn peek(&self, address: Address) -> Value {
        match address {
            Address::Local(lv) => self.machine[lv],
            Address::Constant(v) => v,
            Address::Field(o, f, _) => match &self.machine[o] {
                ObjectData::Instance(i) => i.fields[f],
                ObjectData::Tuple(v) => v.fields[f],
                d => panic!("unexpected thing with fields: {d:?}"),
            },
        }
    }

    /// Overwrites the value at `address`; does not adjust permissions at all.
    pub(super) fn poke(&mut self, address: Address, value: Value) -> eyre::Result<()> {
        match address {
            Address::Local(lv) => self.machine[lv] = value,
            Address::Constant(_) => {
                return Err(error!(
                    self.machine.pc().span(self.db),
                    "cannot store into a constant"
                )
                .eyre(self.db))
            }
            Address::Field(o, f, _) => match &mut self.machine[o] {
                ObjectData::Instance(i) => i.fields[f] = value,
                ObjectData::Tuple(v) => v.fields[f] = value,
                d => panic!("unexpected thing with fields: {d:?}"),
            },
        }
        Ok(())
    }
}
