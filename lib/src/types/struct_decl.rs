use std::fmt::Display;

use crate::{
    Env, Field, TypeKind, TypePath, Types,
    error::{ExnExt as _, OptionExt as _, ResultExt as _, bail_str, error_type},
};

error_type!(StructDeclError);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructDecl {
    pub(crate) path: Option<TypePath>,
    pub(crate) base_types: Vec<TypePath>,
    pub(crate) fields: Vec<StructField>,
    pub(crate) size: usize,
    pub(crate) alignment: usize,
    pub(crate) is_virtual: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructField {
    /// Offset in bits
    pub(crate) offset: usize,
    pub(crate) field: Field,
}

impl StructDecl {
    pub fn new(
        env: &Env,
        types: &Types,
        path: Option<TypePath>,
        node: &clang::Entity,
    ) -> exn::Result<Self, StructDeclError> {
        let node_kind = node.get_kind();
        if !matches!(node_kind, clang::EntityKind::StructDecl | clang::EntityKind::ClassDecl) {
            bail_str!("Expected StructDecl or ClassDecl, found: {:?}", node);
        }

        let display_path = path.clone().unwrap_or("<anon>".into());

        // Check for invalid fields
        {
            let decl_children = node.get_children();
            let invalid_fields = decl_children
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    c.get_kind() == clang::EntityKind::FieldDecl && c.is_invalid_declaration()
                })
                .collect::<Vec<_>>();
            if !invalid_fields.is_empty() {
                bail_str!(
                    "Invalid fields in {}: {:?}",
                    display_path,
                    invalid_fields
                        .iter()
                        .map(|(i, c)| c.get_name().unwrap_or_else(|| format!("<index#{i}>")))
                        .collect::<Vec<_>>()
                );
            }
        }

        let mut base_types = Vec::new();
        let mut fields = Vec::new();
        let mut is_virtual = false;
        let mut alignment = 1;
        for child in node.get_children() {
            match child.get_kind() {
                clang::EntityKind::BaseSpecifier => {
                    let base_type = child
                        .get_type()
                        .ok_or_raise_str(|| format!("BaseSpecifier without type: {:?}", child))?;
                    let base_decl = base_type.get_declaration().ok_or_raise_str(|| {
                        format!("Record base type without declaration: {:?}", node)
                    })?;
                    let base_type_path = TypePath::from_entity(&base_decl).or_raise_str(|| {
                        format!(
                            "Failed to get type path for base type {}",
                            base_type.get_display_name()
                        )
                    })?;
                    base_types.push(base_type_path.clone());
                    let base_type = types.get(base_type_path.clone()).ok_or_raise_str(|| {
                        format!("Base type {} of {} is not defined", base_type_path, display_path)
                    })?;
                    is_virtual |= base_type.is_virtual(types);
                    alignment = alignment.max(base_type.alignment(types));
                }
                clang::EntityKind::FieldDecl => {
                    let offset = Self::get_offset_of_field(&display_path, &child)?;
                    let field = Field::new(env, types, &child).or_raise_str(|| {
                        format!("Failed to parse AST of field in {}", display_path)
                    })?;
                    alignment = alignment.max(field.kind().alignment(types));
                    fields.push(StructField { field, offset });
                }
                clang::EntityKind::Destructor | clang::EntityKind::Method => {
                    is_virtual |= child.is_virtual_method();
                }
                _ => {}
            }
        }

        if is_virtual {
            alignment = alignment.max(env.word_size().bytes());
        }

        let size = if let Some(last_field) = fields.last() {
            (last_field.offset + last_field.size_bits(types))
                .div_ceil(8)
                .next_multiple_of(alignment)
        } else {
            0
        };

        Ok(Self { path, base_types, fields, size, alignment, is_virtual })
    }

    fn get_offset_of_field(
        struct_path: &TypePath,
        node: &clang::Entity,
    ) -> exn::Result<usize, StructDeclError> {
        node.get_offset_of_field().or_raise_str(|| {
            format!(
                "Failed to get field offset of {} in {}",
                node.get_name().unwrap_or_default(),
                struct_path
            )
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

    pub fn is_virtual(&self) -> bool {
        self.is_virtual
    }

    pub fn replace_template_parameters<Cb>(
        &self,
        _types: &Types,
        _get_param_type: Cb,
    ) -> exn::Result<StructDecl, StructDeclError>
    where
        Cb: Fn(&str) -> Option<TypeKind>,
    {
        bail_str!(
            "Template specialization not implemented for template parameters in structs defined inside the template class",
        );
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

    pub fn size_bits(&self, types: &Types) -> usize {
        self.field.size_bits(types)
    }

    pub fn with_kind(&self, kind: TypeKind) -> Self {
        Self { field: self.field.with_kind(kind), ..self.clone() }
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
