use std::fmt::Display;

use crate::{error::ParseError, types::TypeDecl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    pub(crate) name: String,
    constants: Vec<EnumConstant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumConstant {
    name: String,
    value: i64,
}

impl EnumDecl {
    pub fn new(name: String, node: &clang::Entity) -> Result<Self, ParseError> {
        if node.get_kind() != clang::EntityKind::EnumDecl {
            return Err(ParseError::InvalidAst(format!("Expected EnumDecl, found: {node:?}")));
        }

        let mut constants = Vec::new();
        for child in node.get_children() {
            if child.get_kind() != clang::EntityKind::EnumConstantDecl {
                return Err(ParseError::InvalidAst(format!(
                    "Expected EnumConstantDecl, found: {child:?}"
                )));
            }
            let name = child.get_name().ok_or_else(|| {
                ParseError::InvalidAst(format!("EnumConstantDecl without name: {child:?}"))
            })?;
            let (value, _) = child.get_enum_constant_value().ok_or_else(|| {
                ParseError::InvalidAst(format!("EnumConstantDecl without value: {child:?}"))
            })?;
            constants.push(EnumConstant { name, value });
        }

        Ok(EnumDecl { name, constants })
    }
}

impl TypeDecl for EnumDecl {
    fn is_forward_decl(&self) -> bool {
        false
    }
}

impl Display for EnumDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {{ ", self.name)?;
        for constant in &self.constants {
            writeln!(f, "  {}: {:#x}", constant.name, constant.value)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
