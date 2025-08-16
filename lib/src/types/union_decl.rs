use std::fmt::Display;

use crate::{
    Env, Field, Types,
    error::{
        AlignofSnafu, InvalidAstSnafu, InvalidFieldsSnafu, ParseError, SizeofSnafu,
        UnsupportedEntitySnafu, UnsupportedTypeSnafu,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionDecl {
    pub(crate) name: Option<String>,
    fields: Vec<Field>,
    size: usize,
    alignment: usize,
}

impl UnionDecl {
    pub fn new(
        env: &Env,
        types: &Types,
        name: Option<String>,
        ty: clang::Type,
    ) -> Result<Self, ParseError> {
        if ty.get_kind() != clang::TypeKind::Record {
            return InvalidAstSnafu { message: format!("Expected Record, found: {ty:?}") }.fail();
        }

        let display_name = name.as_deref().unwrap_or("<anon>");

        let record_fields = ty.get_fields().ok_or_else(|| {
            UnsupportedTypeSnafu { message: format!("Record type without fields: {ty:?}") }.build()
        })?;
        if record_fields.is_empty() {
            let declaration = ty.get_declaration().ok_or_else(|| {
                InvalidAstSnafu { message: format!("Record type without declaration: {ty:?}") }
                    .build()
            })?;

            let decl_children = declaration.get_children();
            let invalid_fields = decl_children
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    c.get_kind() == clang::EntityKind::FieldDecl && c.is_invalid_declaration()
                })
                .collect::<Vec<_>>();
            if !invalid_fields.is_empty() {
                return InvalidFieldsSnafu {
                    field_names: invalid_fields
                        .iter()
                        .map(|(i, c)| c.get_name().unwrap_or_else(|| format!("<index#{i}>")))
                        .collect::<Vec<_>>(),
                    struct_name: display_name.to_string(),
                }
                .fail();
            }
        }

        let mut fields = Vec::new();
        for field in &record_fields {
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
                            InvalidAstSnafu {
                                message: format!("FieldDecl without name: {field:?}"),
                            }
                            .build()
                        })?;
                        Some(name)
                    };
                    let field_type = field.get_type().ok_or_else(|| {
                        InvalidAstSnafu { message: format!("FieldDecl without type: {field:?}") }
                            .build()
                    })?;
                    fields.push(Field::new(env, types, field_name, field_type)?);
                }
                _ => {
                    return UnsupportedEntitySnafu {
                        at: format!("union {display_name}"),
                        message: format!(
                            "Unsupported entity kind in union: {:?}",
                            field.get_kind()
                        ),
                    }
                    .fail();
                }
            }
        }

        let size = ty.get_sizeof().or_else(|e| {
            if record_fields.is_empty() {
                Ok(1)
            } else {
                SizeofSnafu { type_name: display_name.to_string(), error: e }.fail()
            }
        })?;
        let alignment = ty.get_alignof().or_else(|e| {
            if record_fields.is_empty() {
                Ok(1)
            } else {
                AlignofSnafu { type_name: display_name.to_string(), error: e }.fail()
            }
        })?;

        Ok(UnionDecl { name, fields, size, alignment })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn alignment(&self) -> usize {
        self.alignment
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name() == Some(name))
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
