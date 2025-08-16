use std::fmt::Display;

use crate::{
    Env, Field, Types,
    error::{
        AlignofSnafu, InvalidAstSnafu, InvalidFieldsSnafu, OffsetofSnafu, ParseError, SizeofSnafu,
        UnsupportedEntitySnafu, UnsupportedTypeSnafu,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDecl {
    pub(crate) name: Option<String>,
    pub(crate) base_types: Vec<String>,
    pub(crate) fields: Vec<StructField>,
    size: usize,
    alignment: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    /// Offset in bits
    offset: usize,
    field: Field,
}

impl StructDecl {
    pub fn new(
        env: &Env,
        types: &Types,
        name: Option<String>,
        ty: clang::Type,
    ) -> Result<Self, ParseError> {
        if ty.get_kind() != clang::TypeKind::Record {
            return InvalidAstSnafu { message: format!("Expected Record, found: {ty:?}") }.fail();
        }

        let mut base_types = Vec::new();
        if let Some(node) = ty.get_declaration() {
            for child in node.get_children() {
                if child.get_kind() != clang::EntityKind::BaseSpecifier {
                    continue;
                }
                let base_type = child.get_type().ok_or_else(|| {
                    InvalidAstSnafu { message: format!("BaseSpecifier without type: {child:?}") }
                        .build()
                })?;
                let base_name = base_type.get_display_name();
                base_types.push(base_name);
            }
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

        let mut fields = Vec::<StructField>::new();
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
                        InvalidAstSnafu { message: format!("Field without type: {field:?}") }
                            .build()
                    })?;
                    let offset = Self::get_offset_of_field(display_name, field)?;
                    fields.push(StructField {
                        offset,
                        field: Field::new(env, types, field_name, field_type)?,
                    });
                }
                _ => {
                    return UnsupportedEntitySnafu {
                        at: format!("struct/class {display_name}"),
                        message: format!(
                            "Unsupported entity kind in struct/class: {:?}",
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

        Ok(Self { name, base_types, fields, size, alignment })
    }

    fn get_offset_of_field(struct_name: &str, node: &clang::Entity) -> Result<usize, ParseError> {
        node.get_offset_of_field().map_err(|e| {
            OffsetofSnafu {
                field_name: node.get_name().unwrap_or_default(),
                struct_name: struct_name.to_string(),
                error: e,
            }
            .build()
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn alignment(&self) -> usize {
        self.alignment
    }

    pub fn is_forward_decl(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn base_types(&self) -> &[String] {
        &self.base_types
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn get_field(&self, name: &str) -> Option<&StructField> {
        self.fields.iter().find(|f| f.name() == Some(name))
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl StructField {
    pub fn offset_bytes(&self) -> usize {
        self.offset / 8
    }

    pub fn name(&self) -> Option<&str> {
        self.field.name()
    }

    pub fn kind(&self) -> &super::TypeKind {
        self.field.kind()
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
            writeln!(f, "  ({:#x}) {}", field.offset_bytes(), field.field)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
