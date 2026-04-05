use std::fmt::Display;

use crate::{
    Env, Field, TypePath, Types,
    error::{ExnExt, OptionExt, bail_str, error_type},
};

error_type!(TemplateClassError);

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn path(&self) -> &TypePath {
        &self.path
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
