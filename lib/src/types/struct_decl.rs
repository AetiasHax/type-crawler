use std::fmt::Display;

use crate::{TypeKind, error::ParseError, types::TypeDecl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDecl {
    pub(crate) name: String,
    base_types: Vec<String>,
    fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    name: String,
    offset: usize,
    kind: TypeKind,
}

impl StructDecl {
    pub fn new(name: String, node: &clang::Entity) -> Result<Self, ParseError> {
        if !matches!(node.get_kind(), clang::EntityKind::StructDecl | clang::EntityKind::ClassDecl)
        {
            return Err(ParseError::InvalidAst(format!(
                "Expected StructDecl or ClassDecl, found: {node:?}"
            )));
        }

        let mut fields = Vec::new();
        let mut base_types = Vec::new();
        for child in node.get_children() {
            match child.get_kind() {
                clang::EntityKind::FieldDecl => {
                    let field_name = child.get_name().ok_or_else(|| {
                        ParseError::InvalidAst(format!("FieldDecl without name: {child:?}"))
                    })?;
                    let offset = child.get_offset_of_field().map_err(|e| ParseError::Offsetof {
                        field_name: field_name.clone(),
                        struct_name: name.clone(),
                        error: e,
                    })?;
                    // let offset = child.get_offset_of_field().ok().unwrap_or(usize::MAX);
                    let field_type = child.get_type().ok_or_else(|| {
                        ParseError::InvalidAst(format!("FieldDecl without type: {child:?}"))
                    })?;
                    let kind = TypeKind::new(field_type)?;
                    fields.push(StructField { name: field_name, offset, kind });
                }
                clang::EntityKind::StructDecl | clang::EntityKind::ClassDecl => {
                    // TODO: Handle nested structs/classes
                }
                clang::EntityKind::UnionDecl => {
                    // TODO: Handle unions
                }
                clang::EntityKind::BaseSpecifier => {
                    let base_type = child.get_type().ok_or_else(|| {
                        ParseError::InvalidAst(format!("BaseSpecifier without type: {child:?}"))
                    })?;
                    let base_name = base_type.get_display_name();
                    base_types.push(base_name);
                }
                clang::EntityKind::Method => {}
                clang::EntityKind::Destructor => {}
                clang::EntityKind::Constructor => {}
                clang::EntityKind::AccessSpecifier => {}
                clang::EntityKind::VarDecl => {}
                _ => {
                    return Err(ParseError::UnsupportedEntity {
                        at: format!("struct/class {name}"),
                        message: format!(
                            "Unsupported child kind in struct/class: {:?}",
                            child.get_kind()
                        ),
                    });
                }
            }
        }

        Ok(StructDecl { name, base_types, fields })
    }
}

impl TypeDecl for StructDecl {
    fn is_forward_decl(&self) -> bool {
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
        write!(f, "{}", self.name)?;
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
            writeln!(f, "  {} @ {:#x}: {:x?}", field.name, field.offset_bytes(), field.kind)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
