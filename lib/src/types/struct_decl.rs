use std::fmt::Display;

use crate::{Env, Field, TypeKind, Types, error::ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDecl {
    pub(crate) name: Option<String>,
    pub(crate) base_types: Vec<String>,
    pub(crate) fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub offset: usize,
    pub field: Field,
}

impl StructDecl {
    pub fn new(
        env: &Env,
        types: &Types,
        name: Option<String>,
        ty: clang::Type,
    ) -> Result<Self, ParseError> {
        if ty.get_kind() != clang::TypeKind::Record {
            return Err(ParseError::InvalidAst(format!("Expected Record, found: {ty:?}")));
        }

        let mut base_types = Vec::new();
        if let Some(node) = ty.get_declaration() {
            for child in node.get_children() {
                if child.get_kind() != clang::EntityKind::BaseSpecifier {
                    continue;
                }
                let base_type = child.get_type().ok_or_else(|| {
                    ParseError::InvalidAst(format!("BaseSpecifier without type: {child:?}"))
                })?;
                let base_name = base_type.get_display_name();
                base_types.push(base_name);
            }
        }

        let display_name = name.as_deref().unwrap_or("<anon>");

        let mut fields = Vec::<StructField>::new();
        let record_fields = ty.get_fields().ok_or_else(|| {
            ParseError::UnsupportedType(format!("Record type without fields: {ty:?}"))
        })?;
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
                        ParseError::InvalidAst(format!("Field without type: {field:?}"))
                    })?;
                    let offset = Self::get_offset_of_field(display_name, &field)?;
                    let kind = TypeKind::new(env, types, field_type)?;
                    fields.push(StructField { offset, field: Field::new(field_name, kind) });
                }
                clang::EntityKind::UnionDecl => {
                    // TODO: Handle unions
                }
                clang::EntityKind::BaseSpecifier => {
                    let base_type = field.get_type().ok_or_else(|| {
                        ParseError::InvalidAst(format!("BaseSpecifier without type: {field:?}"))
                    })?;
                    let base_name = base_type.get_display_name();
                    base_types.push(base_name);
                }
                _ => {
                    return Err(ParseError::UnsupportedEntity {
                        at: format!("struct/class {display_name}"),
                        message: format!(
                            "Unsupported entity kind in struct/class: {:?}",
                            field.get_kind()
                        ),
                    });
                }
            }
        }

        Ok(Self { name, base_types, fields })
    }

    fn get_offset_of_field(struct_name: &str, node: &clang::Entity) -> Result<usize, ParseError> {
        node.get_offset_of_field().map_err(|e| ParseError::Offsetof {
            field_name: node.get_name().unwrap_or_default(),
            struct_name: struct_name.to_string(),
            error: e,
        })
    }

    pub fn size(&self, env: &Env, types: &Types) -> Option<usize> {
        if self.fields.is_empty() {
            Some(0)
        } else {
            let last_field = self.fields.last().unwrap();
            if let Some(last_field_size) = last_field.field.kind().size(env, types) {
                Some(last_field.offset + last_field_size)
            } else if self.fields.len() == 1 {
                Some(0)
            } else {
                // Last field could be an incomplete array
                let second_last_field = self.fields.get(self.fields.len() - 2).unwrap();
                let size = second_last_field.field.kind().size(env, types)?;
                Some(second_last_field.offset + size)
            }
        }
    }

    pub fn alignment(&self, env: &Env, types: &Types) -> Option<usize> {
        let mut alignment = 1;
        for field in &self.fields {
            let field_alignment = field.field.kind().alignment(env, types)?;
            alignment = alignment.max(field_alignment);
        }
        Some(alignment)
    }

    pub fn is_forward_decl(&self) -> bool {
        self.fields.is_empty()
    }
}

impl StructField {
    pub fn offset_bytes(&self) -> usize {
        self.offset / 8
    }
}

impl Display for StructDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.as_deref().unwrap_or("<anon>"))?;
        if !self.base_types.is_empty() {
            write!(f, " : ")?;
            let mut iter = self.base_types.iter();
            write!(f, "{}", iter.next().unwrap())?;
            for base in iter {
                write!(f, ", {base}")?;
            }
        }
        writeln!(f, " {{")?;
        for field in &self.fields {
            let name = field.field.name().unwrap_or("<anon>");
            writeln!(f, "  {} @ {:#x}: {:x?}", name, field.offset_bytes(), field.field.kind())?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
