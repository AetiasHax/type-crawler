use std::fmt::Display;

use crate::error::{InvalidAstSnafu, ParseError, SizeofSnafu};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    pub(crate) name: Option<String>,
    constants: Vec<EnumConstant>,
    size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumConstant {
    name: String,
    value: i64,
}

impl EnumDecl {
    pub fn new(name: Option<String>, node: &clang::Entity) -> Result<Self, ParseError> {
        if node.get_kind() != clang::EntityKind::EnumDecl {
            return InvalidAstSnafu { message: format!("Expected EnumDecl, found: {node:?}") }
                .fail();
        }

        let underlying_type = node.get_enum_underlying_type().ok_or_else(|| {
            InvalidAstSnafu { message: format!("EnumDecl without underlying type: {node:?}") }
                .build()
        })?;
        let size = underlying_type.get_sizeof().map_err(|e| {
            SizeofSnafu { type_name: underlying_type.get_display_name(), error: e }.build()
        })?;

        let mut constants = Vec::new();
        for child in node.get_children() {
            if child.get_kind() != clang::EntityKind::EnumConstantDecl {
                return InvalidAstSnafu {
                    message: format!("Expected EnumConstantDecl, found: {child:?}"),
                }
                .fail();
            }
            let name = child.get_name().ok_or_else(|| {
                { InvalidAstSnafu { message: format!("EnumConstantDecl without name: {child:?}") } }
                    .build()
            })?;
            let (value, _) = child.get_enum_constant_value().ok_or_else(|| {
                InvalidAstSnafu { message: format!("EnumConstantDecl without value: {child:?}") }
                    .build()
            })?;
            constants.push(EnumConstant { name, value });
        }

        Ok(EnumDecl { name, constants, size })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn alignment(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn get(&self, value: i64) -> Option<&EnumConstant> {
        self.constants.iter().find(|c| c.value == value)
    }

    pub fn constants(&self) -> &[EnumConstant] {
        &self.constants
    }
}

impl EnumConstant {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

impl Display for EnumDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} (size={}) {{ ", self.name.as_deref().unwrap_or("<anon>"), self.size)?;
        for constant in &self.constants {
            writeln!(f, "  {}: {:#x}", constant.name, constant.value)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
