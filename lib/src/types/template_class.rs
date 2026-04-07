use std::fmt::Display;

use crate::{
    Env, Field, StructDecl, StructField, TypeKind, TypePath, Types,
    error::{ExnExt, OptionExt, bail_str, ensure_str, error_type},
};

error_type!(TemplateClassError);

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateClass {
    path: TypePath,
    parameters: Vec<String>,
    base_types: Vec<TypePath>,
    fields: Vec<Field>,
    is_virtual: bool,
}

impl TemplateClass {
    pub(crate) fn new(
        env: &Env,
        types: &Types,
        path: TypePath,
        node: &clang::Entity,
    ) -> exn::Result<Self, TemplateClassError> {
        let node_kind = node.get_kind();
        if node_kind != clang::EntityKind::ClassTemplate {
            bail_str!("Expected ClassTemplate, found: {:?}", node);
        }

        let mut base_types = Vec::new();
        let mut fields = Vec::new();
        let mut is_virtual = false;
        let mut parameters = Vec::new();
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
                            "Failed to get path to base type {} of template class {}",
                            base_type.get_display_name(),
                            path
                        )
                    })?;
                    base_types.push(base_type_path.clone());
                    let base_type = types.get(base_type_path.clone()).ok_or_raise_str(|| {
                        format!("Base type {} of {} is not defined", base_type_path, path)
                    })?;
                    is_virtual |= base_type.is_virtual(types);
                }
                clang::EntityKind::FieldDecl => {
                    let field = Field::new(env, types, &child).or_raise_str(|| {
                        format!("Failed to parse AST for field in template class {}", path)
                    })?;
                    fields.push(field);
                }
                clang::EntityKind::Destructor | clang::EntityKind::Method => {
                    is_virtual |= child.is_virtual_method();
                }
                clang::EntityKind::TemplateTypeParameter => {
                    let name = child
                        .get_display_name()
                        .ok_or_raise_str(|| "TemplateTypeParameter without a name")?;
                    parameters.push(name);
                }
                _ => {}
            }
        }

        Ok(Self { path, parameters, base_types, fields, is_virtual })
    }

    pub fn specialize(
        &self,
        env: &Env,
        types: &Types,
        template_args: &[TypeKind],
    ) -> exn::Result<StructDecl, TemplateClassError> {
        // Verify number of template arguments
        ensure_str!(
            template_args.len() == self.parameters.len(),
            "Expected {} arguments during specialization of template class {} but got {}",
            self.parameters.len(),
            self.path,
            self.parameters.len()
        );

        // Determine offset of first field
        let mut offset = 0;
        let mut base_is_virtual = false;
        let mut alignment = 1;
        for base_type_path in &self.base_types {
            let base_type = types.get(base_type_path.clone()).ok_or_raise_str(|| {
                format!(
                    "Failed to find base type during specialization of template class {}",
                    self.path
                )
            })?;
            ensure_str!(
                base_type.size(types) > 0,
                "Base type {} of template class {} has size zero. {} may be inheriting from another template class, which is not supported currently.",
                base_type_path,
                self.path,
                self.path
            );
            offset += base_type.size(types);
            base_is_virtual |= base_type.is_virtual(types);
            alignment = alignment.max(base_type.alignment(types));
        }
        if self.is_virtual && !base_is_virtual {
            // Virtual table
            let pointer_size = env.word_size().bytes();
            offset += pointer_size;
            alignment = alignment.max(pointer_size);
        }

        // Convert template class's fields to struct fields
        let fields = self
            .fields
            .iter()
            .map(|field| -> exn::Result<StructField, TemplateClassError> {
                let kind = field.kind().replace_template_parameters(types, |param| {
                    let param_index = self.parameters.iter().position(|p| p == param)?;
                    Some(template_args[param_index].clone())
                }).or_raise_str(|| format!("Failed to replace template parameters inside field {:?} of template class {}", field.name, self.path))?;
                let kind_alignment = kind.alignment(types);
                alignment = alignment.max(kind_alignment);
                let spec_field = field.with_kind(kind.clone());
                if field.bit_field_width().is_none() {
                    // Align if not bit field
                    offset = offset.next_multiple_of(kind_alignment * 8);
                }
                let struct_field = StructField { offset, field: spec_field };
                offset += struct_field.size_bits(types);
                Ok(struct_field)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let size = offset.next_multiple_of(alignment);

        Ok(StructDecl {
            path: Some(self.path.with_template_arguments(template_args.to_vec())),
            base_types: self.base_types.clone(),
            fields,
            size,
            alignment,
            is_virtual: self.is_virtual,
        })
    }

    pub fn path(&self) -> &TypePath {
        &self.path
    }

    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn base_types(&self) -> &[TypePath] {
        &self.base_types
    }

    pub fn is_virtual(&self) -> bool {
        self.is_virtual
    }
}

impl Display for TemplateClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}<", self.path)?;
        if !self.parameters.is_empty() {
            let mut iter = self.parameters.iter();
            write!(f, "{}", iter.next().unwrap())?;
            for parameter in iter {
                write!(f, ", {parameter}")?;
            }
        }
        write!(f, ">")?;

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
            writeln!(f, "  {field}")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
