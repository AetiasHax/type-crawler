mod enum_decl;
mod field;
mod struct_decl;
mod type_kind;
mod typedef;
mod union_decl;

use indexmap::IndexMap;

pub use enum_decl::{EnumConstant, EnumDecl};
pub use field::Field;
use snafu::Snafu;
pub use struct_decl::{StructDecl, StructField};
pub use type_kind::TypeKind;
pub use typedef::Typedef;
pub use union_decl::UnionDecl;

#[derive(Default)]
pub struct Types {
    types: IndexMap<String, TypeKind>,
}

#[derive(Debug, Snafu)]
pub enum ExtendTypesError {
    #[snafu(display("Type with the same name but different definitions:\n{left}\nand\n{right}"))]
    ConflictingTypes { left: Box<TypeKind>, right: Box<TypeKind> },
}

impl Types {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_type(&mut self, kind: TypeKind) -> bool {
        if let Some(name) = kind.name() {
            self.types.insert(name.to_string(), kind);
            true
        } else {
            false
        }
    }

    pub fn types(&self) -> impl Iterator<Item = &TypeKind> {
        self.types.values()
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, name: &str) -> Option<&TypeKind> {
        self.types.get(name)
    }

    pub fn extend(&mut self, other: Types) -> Result<(), ExtendTypesError> {
        for (name, value) in other.types {
            match self.types.entry(name.clone()) {
                indexmap::map::Entry::Occupied(mut entry) => {
                    let current = entry.get();
                    if current.is_forward_decl() {
                        entry.insert(value);
                    } else if !value.is_forward_decl() && current != &value {
                        return ConflictingTypesSnafu {
                            left: Box::new(current.clone()),
                            right: Box::new(value),
                        }
                        .fail();
                    }
                }
                indexmap::map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }
        Ok(())
    }
}
