use std::fmt::Display;

use crate::{Env, TypePath, Types, error::ParseError, types::TypeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Typedef {
    path: TypePath,
    underlying_type: TypeKind,
    constant: bool,
    volatile: bool,
}

impl Typedef {
    pub fn new(
        env: &Env,
        types: &Types,
        path: TypePath,
        underlying_type: clang::Type,
    ) -> Result<Self, ParseError> {
        Ok(Typedef {
            path,
            underlying_type: TypeKind::new(env, types, underlying_type)?,
            constant: underlying_type.is_const_qualified(),
            volatile: underlying_type.is_volatile_qualified(),
        })
    }

    pub fn underlying_type(&self) -> &TypeKind {
        &self.underlying_type
    }

    #[deprecated(note = "use path().name() instead")]
    pub fn name(&self) -> &str {
        self.path.name()
    }

    pub fn path(&self) -> &TypePath {
        &self.path
    }

    pub fn constant(&self) -> bool {
        self.constant
    }

    pub fn volatile(&self) -> bool {
        self.volatile
    }
}

impl Display for Typedef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "typedef {}{}{:?} {}",
            if self.constant { "const " } else { "" },
            if self.volatile { "volatile " } else { "" },
            self.underlying_type,
            self.path,
        )
    }
}
