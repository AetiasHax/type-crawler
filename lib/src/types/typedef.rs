use std::fmt::Display;

use crate::{Env, Types, error::ParseError, types::TypeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typedef {
    pub(crate) name: String,
    underlying_type: TypeKind,
    constant: bool,
    volatile: bool,
}

impl Typedef {
    pub fn new(
        env: &Env,
        types: &Types,
        name: String,
        underlying_type: clang::Type,
    ) -> Result<Self, ParseError> {
        Ok(Typedef {
            name,
            underlying_type: TypeKind::new(env, types, underlying_type)?,
            constant: underlying_type.is_const_qualified(),
            volatile: underlying_type.is_volatile_qualified(),
        })
    }

    pub fn underlying_type(&self) -> &TypeKind {
        &self.underlying_type
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
