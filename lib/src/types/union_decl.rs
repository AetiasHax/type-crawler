use std::fmt::Display;

use crate::{Field, TypeKind, error::ParseError, types::TypeDecl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionDecl {
    pub(crate) name: String,
    fields: Vec<Field>,
}

impl UnionDecl {
    pub fn new(name: String, node: &clang::Entity) -> Result<Self, ParseError> {
        if node.get_kind() != clang::EntityKind::UnionDecl {
            return Err(ParseError::InvalidAst(format!("Expected UnionDecl, found: {node:?}")));
        }

        let mut fields = Vec::new();
        for child in node.get_children() {
            match child.get_kind() {
                clang::EntityKind::FieldDecl => {
                    let field_name = child.get_name().ok_or_else(|| {
                        ParseError::InvalidAst(format!("FieldDecl without name: {child:?}"))
                    })?;
                    let field_type = child.get_type().ok_or_else(|| {
                        ParseError::InvalidAst(format!("FieldDecl without type: {child:?}"))
                    })?;
                    let kind = TypeKind::new(field_type)?;
                    fields.push(Field::new(field_name, kind));
                }
                _ => {
                    return Err(ParseError::UnsupportedEntity {
                        at: format!("union {name}"),
                        message: format!(
                            "Unsupported entity kind in union: {:?}",
                            child.get_kind()
                        ),
                    });
                }
            }
        }

        Ok(UnionDecl { name, fields })
    }
}

impl TypeDecl for UnionDecl {
    fn is_forward_decl(&self) -> bool {
        false
    }
}

impl Display for UnionDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {{", self.name)?;
        for field in &self.fields {
            writeln!(f, "  {field}")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
