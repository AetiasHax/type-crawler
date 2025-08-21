use std::fmt::Display;

use crate::{
    Env, TypeKind, Types,
    error::{InvalidAstSnafu, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: Option<String>,
    kind: TypeKind,
    constant: bool,
    volatile: bool,
    bit_field_width: Option<u8>,
}

impl Field {
    pub fn new(env: &Env, types: &Types, field: &clang::Entity) -> Result<Self, ParseError> {
        if field.get_kind() != clang::EntityKind::FieldDecl {
            return InvalidAstSnafu { message: format!("Expected FieldDecl, found: {field:?}") }
                .fail();
        }

        let anonymous = if let Some(child) = field.get_child(0) {
            child.is_anonymous()
        } else {
            false
        };

        let name = if anonymous {
            None
        } else {
            let name = field.get_name().ok_or_else(|| {
                InvalidAstSnafu { message: format!("FieldDecl without name: {field:?}") }.build()
            })?;
            Some(name)
        };
        let ty = field.get_type().ok_or_else(|| {
            InvalidAstSnafu { message: format!("Field without type: {field:?}") }.build()
        })?;

        let kind = TypeKind::new(env, types, ty)?;
        let constant = ty.is_const_qualified();
        let volatile = ty.is_volatile_qualified();
        let bit_field_width = field.get_bit_field_width().map(|w| w as u8);
        Ok(Self { name, kind, constant, volatile, bit_field_width })
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }

    pub fn constant(&self) -> bool {
        self.constant
    }

    pub fn volatile(&self) -> bool {
        self.volatile
    }

    pub fn bit_field_width(&self) -> Option<u8> {
        self.bit_field_width
    }

    pub fn size(&self, types: &Types) -> usize {
        self.bit_field_width()
            .map(|w| w.div_ceil(8) as usize)
            .unwrap_or_else(|| self.kind.size(types))
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
