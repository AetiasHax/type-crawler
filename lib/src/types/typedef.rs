use std::fmt::Display;

use crate::{
    error::ParseError,
    types::{TypeDecl, TypeKind},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Typedef {
    pub(crate) name: String,
    underlying_type: TypeKind,
    constant: bool,
    volatile: bool,
}

impl Typedef {
    pub fn new(name: String, underlying_type: clang::Type) -> Result<Self, ParseError> {
        Ok(Typedef {
            name,
            underlying_type: TypeKind::new(underlying_type)?,
            constant: underlying_type.is_const_qualified(),
            volatile: underlying_type.is_volatile_qualified(),
        })
    }
}

impl TypeDecl for Typedef {
    fn is_forward_decl(&self) -> bool {
        false
    }
}

impl Display for Typedef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}{}{:?}",
            self.name,
            if self.constant { "const " } else { "" },
            if self.volatile { "volatile " } else { "" },
            self.underlying_type
        )
    }
}
