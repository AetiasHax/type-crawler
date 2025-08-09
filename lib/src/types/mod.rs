mod enum_decl;
mod field;
mod struct_decl;
mod type_kind;
mod typedef;
mod union_decl;

use std::fmt::Display;

use indexmap::IndexMap;
use thiserror::Error;

pub use enum_decl::{EnumConstant, EnumDecl};
pub use field::Field;
pub use struct_decl::{StructDecl, StructField};
pub use type_kind::TypeKind;
pub use typedef::Typedef;
pub use union_decl::UnionDecl;

use crate::Env;

#[derive(Default)]
pub struct Types {
    types: IndexMap<String, TypeDecl>,
}

#[derive(Error, Debug)]
pub enum ExtendTypesError {
    #[error("Type with the same name but different definitions: {0} and {1}")]
    ConflictingTypes(Box<TypeDecl>, Box<TypeDecl>),
}

impl Types {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_type(&mut self, type_decl: TypeDecl) -> bool {
        if let Some(name) = type_decl.name() {
            self.types.insert(name.clone(), type_decl);
            true
        } else {
            false
        }
    }

    pub fn types(&self) -> impl Iterator<Item = &TypeDecl> {
        self.types.values()
    }

    pub fn get(&self, name: &str) -> Option<&TypeDecl> {
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
                        return Err(ExtendTypesError::ConflictingTypes(
                            Box::new(current.clone()),
                            Box::new(value),
                        ));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDecl {
    Typedef(Typedef),
    Enum(EnumDecl),
    Struct(StructDecl),
    Union(UnionDecl),
}

impl TypeDecl {
    pub fn name(&self) -> Option<&String> {
        match self {
            TypeDecl::Typedef(typedef) => Some(&typedef.name),
            TypeDecl::Enum(enum_decl) => Some(&enum_decl.name),
            TypeDecl::Struct(struct_decl) => struct_decl.name.as_ref(),
            TypeDecl::Union(union_decl) => Some(&union_decl.name),
        }
    }

    pub fn is_forward_decl(&self) -> bool {
        match self {
            TypeDecl::Typedef(_typedef) => false,
            TypeDecl::Enum(_enum_decl) => false,
            TypeDecl::Struct(struct_decl) => struct_decl.is_forward_decl(),
            TypeDecl::Union(_union_decl) => false,
        }
    }

    pub fn size(&self, env: &Env, types: &Types) -> Option<usize> {
        match self {
            TypeDecl::Typedef(typedef) => typedef.underlying_type().size(env, types),
            TypeDecl::Enum(_enum_decl) => Some(4), // TODO: Handle enum size properly
            TypeDecl::Struct(struct_decl) => struct_decl.size(env, types),
            TypeDecl::Union(union_decl) => union_decl.size(env, types),
        }
    }

    pub fn alignment(&self, env: &Env, types: &Types) -> Option<usize> {
        match self {
            TypeDecl::Typedef(typedef) => typedef.underlying_type().alignment(env, types),
            TypeDecl::Enum(_enum_decl) => Some(4), // TODO: Handle enum alignment properly
            TypeDecl::Struct(struct_decl) => struct_decl.alignment(env, types),
            TypeDecl::Union(union_decl) => union_decl.alignment(env, types),
        }
    }
}

impl Display for TypeDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeDecl::Typedef(typedef) => write!(f, "Typedef: {typedef}"),
            TypeDecl::Enum(enum_decl) => write!(f, "Enum: {enum_decl}"),
            TypeDecl::Struct(struct_decl) => write!(f, "Struct: {struct_decl}"),
            TypeDecl::Union(union_decl) => write!(f, "Union: {union_decl}"),
        }
    }
}
