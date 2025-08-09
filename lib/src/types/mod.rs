mod enum_decl;
mod field;
mod struct_decl;
mod type_kind;
mod typedef;
mod union_decl;

use indexmap::IndexMap;
use thiserror::Error;

pub use enum_decl::{EnumConstant, EnumDecl};
pub use field::Field;
pub use struct_decl::{StructDecl, StructField};
pub use type_kind::TypeKind;
pub use typedef::Typedef;
pub use union_decl::UnionDecl;

#[derive(Default)]
pub struct Types {
    typedefs: IndexMap<String, Typedef>,
    enums: IndexMap<String, EnumDecl>,
    structs: IndexMap<String, StructDecl>,
    unions: IndexMap<String, UnionDecl>,
}

#[derive(Error, Debug)]
pub enum ExtendTypesError {
    #[error("Typedefs with the same name but different underlying types: {0} and {1}")]
    ConflictingTypedefs(Box<Typedef>, Box<Typedef>),
    #[error("Enums with the same name but different definitions: {0} and {1}")]
    ConflictingEnums(Box<EnumDecl>, Box<EnumDecl>),
    #[error("Structs with the same name but different definitions: {0} and {1}")]
    ConflictingStructs(Box<StructDecl>, Box<StructDecl>),
    #[error("Unions with the same name but different definitions: {0} and {1}")]
    ConflictingUnions(Box<UnionDecl>, Box<UnionDecl>),
}

impl Types {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_typedef(&mut self, typedef: Typedef) {
        self.typedefs.insert(typedef.name.clone(), typedef);
    }

    pub fn typedefs(&self) -> impl Iterator<Item = &Typedef> {
        self.typedefs.values()
    }

    pub fn add_enum(&mut self, enum_decl: EnumDecl) {
        self.enums.insert(enum_decl.name.clone(), enum_decl);
    }

    pub fn enums(&self) -> impl Iterator<Item = &EnumDecl> {
        self.enums.values()
    }

    pub fn add_struct(&mut self, struct_decl: StructDecl) {
        self.structs.insert(struct_decl.name.clone(), struct_decl);
    }

    pub fn structs(&self) -> impl Iterator<Item = &StructDecl> {
        self.structs.values()
    }

    pub fn add_union(&mut self, union_decl: UnionDecl) {
        self.unions.insert(union_decl.name.clone(), union_decl);
    }

    pub fn unions(&self) -> impl Iterator<Item = &UnionDecl> {
        self.unions.values()
    }

    fn extend_map<V, ErrCb>(
        map: &mut IndexMap<String, V>,
        other: IndexMap<String, V>,
        err_cb: ErrCb,
    ) -> Result<(), ExtendTypesError>
    where
        V: PartialEq + Clone + TypeDecl,
        ErrCb: Fn(Box<V>, Box<V>) -> ExtendTypesError,
    {
        for (name, value) in other {
            match map.entry(name.clone()) {
                indexmap::map::Entry::Occupied(mut entry) => {
                    let current = entry.get();
                    if current.is_forward_decl() {
                        entry.insert(value);
                    } else if !value.is_forward_decl() && current != &value {
                        return Err(err_cb(Box::new(current.clone()), Box::new(value)));
                    }
                }
                indexmap::map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }
        Ok(())
    }

    pub fn extend(&mut self, other: Types) -> Result<(), ExtendTypesError> {
        Self::extend_map(
            &mut self.typedefs,
            other.typedefs,
            ExtendTypesError::ConflictingTypedefs,
        )?;
        Self::extend_map(&mut self.enums, other.enums, ExtendTypesError::ConflictingEnums)?;
        Self::extend_map(&mut self.structs, other.structs, ExtendTypesError::ConflictingStructs)?;
        Self::extend_map(&mut self.unions, other.unions, ExtendTypesError::ConflictingUnions)?;
        Ok(())
    }
}

trait TypeDecl {
    fn is_forward_decl(&self) -> bool;
}
