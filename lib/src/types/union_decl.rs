use std::fmt::Display;

use crate::{Env, Field, TypeKind, Types, error::ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionDecl {
    pub(crate) name: Option<String>,
    fields: Vec<Field>,
}

impl UnionDecl {
    pub fn new(
        env: &Env,
        types: &Types,
        name: Option<String>,
        ty: clang::Type,
    ) -> Result<Self, ParseError> {
        if ty.get_kind() != clang::TypeKind::Record {
            return Err(ParseError::InvalidAst(format!("Expected Record, found: {ty:?}")));
        }

        let record_fields = ty.get_fields().ok_or_else(|| {
            ParseError::UnsupportedType(format!("Record type without fields: {ty:?}"))
        })?;

        let display_name = name.as_deref().unwrap_or("<anon>");

        let mut fields = Vec::new();
        for field in record_fields {
            match field.get_kind() {
                clang::EntityKind::FieldDecl => {
                    let anonymous = if let Some(child) = field.get_child(0) {
                        child.is_anonymous()
                    } else {
                        false
                    };
                    let field_name = if anonymous {
                        None
                    } else {
                        let name = field.get_name().ok_or_else(|| {
                            ParseError::InvalidAst(format!("FieldDecl without name: {field:?}"))
                        })?;
                        Some(name)
                    };
                    let field_type = field.get_type().ok_or_else(|| {
                        ParseError::InvalidAst(format!("FieldDecl without type: {field:?}"))
                    })?;
                    let kind = TypeKind::new(env, types, field_type)?;
                    fields.push(Field::new(field_name, kind));
                }
                _ => {
                    return Err(ParseError::UnsupportedEntity {
                        at: format!("union {display_name}"),
                        message: format!(
                            "Unsupported entity kind in union: {:?}",
                            field.get_kind()
                        ),
                    });
                }
            }
        }

        Ok(UnionDecl { name, fields })
    }

    pub fn size(&self, env: &Env, types: &Types) -> Option<usize> {
        let mut size = 0;
        for field in &self.fields {
            let field_size = field.kind().size(env, types)?;
            size = size.max(field_size);
        }
        Some(size)
    }

    pub fn alignment(&self, env: &Env, types: &Types) -> Option<usize> {
        let mut alignment = 1;
        for field in &self.fields {
            let field_alignment = field.kind().alignment(env, types)?;
            alignment = alignment.max(field_alignment);
        }
        Some(alignment)
    }
}

impl Display for UnionDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {{", self.name.as_deref().unwrap_or("<anon>"))?;
        for field in &self.fields {
            writeln!(f, "  {field}")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
