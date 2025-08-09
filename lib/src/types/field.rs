use std::fmt::Display;

use crate::{Env, TypeKind, Types, error::ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: Option<String>,
    kind: TypeKind,
    constant: bool,
    volatile: bool,
}

impl Field {
    pub fn new(
        env: &Env,
        types: &Types,
        name: Option<String>,
        ty: clang::Type,
    ) -> Result<Self, ParseError> {
        let kind = TypeKind::new(env, types, ty)?;
        let constant = ty.is_const_qualified();
        let volatile = ty.is_volatile_qualified();
        Ok(Self { name, kind, constant, volatile })
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.as_deref().unwrap_or("<anon>");
        write!(
            f,
            "{}: {}{}{:?}",
            name,
            if self.constant { "const " } else { "" },
            if self.volatile { "volatile " } else { "" },
            self.kind
        )
    }
}
