use std::fmt::Display;

use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    pub(crate) name: String,
    constants: Vec<EnumConstant>,
    size: usize,
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

        let underlying_type = node.get_enum_underlying_type().ok_or_else(|| {
            ParseError::InvalidAst(format!("EnumDecl without underlying type: {node:?}"))
        })?;
        let size = underlying_type.get_sizeof()?;

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

        Ok(EnumDecl { name, constants, size })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn alignment(&self) -> usize {
        self.size
    }
}

impl Display for EnumDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} (size={}) {{ ", self.name, self.size)?;
        for constant in &self.constants {
            writeln!(f, "  {}: {:#x}", constant.name, constant.value)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
