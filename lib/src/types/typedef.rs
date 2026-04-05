use std::fmt::Display;

use crate::{
    Env, TypePath, Types,
    error::{ExnExt, error_type},
    types::TypeKind,
};

error_type!(TypedefError);

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
    ) -> exn::Result<Self, TypedefError> {
        let constant = underlying_type.is_const_qualified();
        let volatile = underlying_type.is_volatile_qualified();
        let underlying_type = TypeKind::new(env, types, underlying_type)
            .or_raise_str(|| format!("Failed to get underlying type for typedef '{}'", path))?;
        Ok(Typedef { path, underlying_type, constant, volatile })
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
