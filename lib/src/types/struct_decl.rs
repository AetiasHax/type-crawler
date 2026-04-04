use std::fmt::Display;

use crate::{
    Env, Field, TypeKind, TypePath, Types,
    error::{
        AlignofSnafu, InvalidAstSnafu, InvalidFieldsSnafu, OffsetofSnafu, ParseError, SizeofSnafu,
        UnsupportedEntitySnafu, UnsupportedTypeSnafu,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructDecl {
    pub(crate) path: Option<TypePath>,
    pub(crate) base_types: Vec<TypePath>,
    pub(crate) fields: Vec<StructField>,
    size: usize,
    alignment: usize,
    is_class: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructField {
    /// Offset in bits
    offset: usize,
    field: Field,
}

impl StructDecl {
    pub fn new(
        env: &Env,
        types: &Types,
        path: Option<TypePath>,
        ty: clang::Type,
    ) -> Result<Self, ParseError> {
        if ty.get_kind() != clang::TypeKind::Record {
            return InvalidAstSnafu { message: format!("Expected Record, found: {ty:?}") }.fail();
        }

        let mut base_types = Vec::new();
        let Some(node) = ty.get_declaration() else {
            return InvalidAstSnafu { message: format!("Record type without declaration: {ty:?}") }
                .fail();
        };
        for child in node.get_children() {
            if child.get_kind() != clang::EntityKind::BaseSpecifier {
                continue;
            }
            let base_type = child.get_type().ok_or_else(|| {
                InvalidAstSnafu { message: format!("BaseSpecifier without type: {child:?}") }
                    .build()
            })?;
            let base_decl = base_type.get_declaration().ok_or_else(|| {
                UnsupportedTypeSnafu {
                    message: format!("Record base type without declaration: {ty:?}"),
                }
                .build()
            })?;
            base_types.push(TypePath::from_entity(&base_decl)?);
        }

        let is_class = node.get_kind() == clang::EntityKind::ClassDecl;

        let display_path = path.clone().unwrap_or("<anon>".into());

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
                    struct_name: display_path.to_string(),
                }
                .fail();
            }
        }

        let mut fields = Vec::<StructField>::new();
        for field in &record_fields {
            match field.get_kind() {
                clang::EntityKind::FieldDecl => {
                    let offset = Self::get_offset_of_field(&display_path, field)?;
                    fields.push(StructField { offset, field: Field::new(env, types, field)? });
                }
                _ => {
                    return UnsupportedEntitySnafu {
                        at: format!("struct/class {display_path}"),
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
                SizeofSnafu { type_name: display_path.to_string(), error: e }.fail()
            }
        })?;
        let alignment = ty.get_alignof().or_else(|e| {
            if record_fields.is_empty() {
                Ok(1)
            } else {
                AlignofSnafu { type_name: display_path.to_string(), error: e }.fail()
            }
        })?;

        Ok(Self { path, base_types, fields, size, alignment, is_class })
    }

    fn get_offset_of_field(
        struct_path: &TypePath,
        node: &clang::Entity,
    ) -> Result<usize, ParseError> {
        node.get_offset_of_field().map_err(|e| {
            OffsetofSnafu {
                field_name: node.get_name().unwrap_or_default(),
                struct_path: struct_path.clone(),
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

    pub fn base_types(&self) -> &[TypePath] {
        &self.base_types
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn get_field<'a>(&'a self, types: &'a Types, name: &str) -> Option<&'a StructField> {
        self.fields.iter().find(|f| f.name() == Some(name)).or_else(|| {
            self.base_types
                .iter()
                .filter_map(|base| types.get(base.clone()))
                .filter_map(|base| base.expand_named(types))
                .filter_map(|base| match base {
                    TypeKind::Struct(struct_decl) => struct_decl.get_field(types, name),
                    TypeKind::Class(class_decl) => class_decl.get_field(types, name),
                    _ => None,
                })
                .next()
        })
    }

    #[deprecated(note = "use path().map(|p| p.name()) instead")]
    pub fn name(&self) -> Option<&str> {
        self.path.as_ref().map(|p| p.name())
    }

    pub fn path(&self) -> Option<&TypePath> {
        self.path.as_ref()
    }

    pub fn is_class(&self) -> bool {
        self.is_class
    }
}

impl StructField {
    pub fn offset_bytes(&self) -> usize {
        self.offset / 8
    }

    pub fn offset_bits(&self) -> usize {
        self.offset
    }

    pub fn name(&self) -> Option<&str> {
        self.field.name()
    }

    pub fn kind(&self) -> &super::TypeKind {
        self.field.kind()
    }

    pub fn constant(&self) -> bool {
        self.field.constant()
    }

    pub fn volatile(&self) -> bool {
        self.field.volatile()
    }

    pub fn bit_field_width(&self) -> Option<u8> {
        self.field.bit_field_width()
    }

    pub fn size(&self, types: &Types) -> usize {
        self.field.size(types)
    }
}

impl Display for StructDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.clone().unwrap_or("<anon>".into()))?;
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
