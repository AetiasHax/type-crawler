use std::{borrow::Cow, fmt::Display};

use crate::{
    TypeKind,
    error::{OptionExt as _, error_type},
};

error_type!(TypePathError);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypePath {
    namespaces: Vec<String>,
    name: String,
    template_arguments: Vec<TypeKind>,
}

impl TypePath {
    pub fn new(
        namespaces: Vec<String>,
        name: impl Into<String>,
        template_arguments: Vec<TypeKind>,
    ) -> Self {
        Self { namespaces, name: name.into(), template_arguments }
    }

    pub fn global(name: impl Into<String>) -> Self {
        Self { name: name.into(), namespaces: Vec::new(), template_arguments: Vec::new() }
    }

    pub fn from_entity(entity: &clang::Entity) -> exn::Result<Self, TypePathError> {
        let mut namespaces = Vec::new();
        let mut current = entity.get_semantic_parent();
        while let Some(parent) = current {
            if parent.get_kind() == clang::EntityKind::Namespace
                && let Some(name) = parent.get_name()
            {
                namespaces.push(name);
            }
            current = parent.get_semantic_parent();
        }
        namespaces.reverse(); // Outermost to innermost

        let name = entity.get_name().ok_or_raise_str(|| {
            format!("Elaborated type declaration without name: {:?}", entity)
        })?;

        // Template arguments are only ever filled in by `with_template_arguments`
        Ok(Self { namespaces, name, template_arguments: Vec::new() })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn namespaces(&self) -> &[String] {
        &self.namespaces
    }

    pub fn with_template_arguments(&self, template_arguments: Vec<TypeKind>) -> Self {
        Self { namespaces: self.namespaces.clone(), name: self.name.clone(), template_arguments }
    }
}

impl From<&str> for TypePath {
    fn from(value: &str) -> Self {
        Self::global(value)
    }
}

impl Display for TypePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for namespace in &self.namespaces {
            write!(f, "{namespace}::")?;
        }
        write!(f, "{}", self.name)?;
        if !self.template_arguments.is_empty() {
            write!(f, "<")?;
            let mut iter = self.template_arguments.iter();
            write!(f, "{}", iter.next().unwrap())?;
            for parameter in iter {
                write!(f, ", {parameter}")?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

impl<'a> From<&'a TypePath> for Cow<'a, TypePath> {
    fn from(val: &'a TypePath) -> Self {
        Cow::Borrowed(val)
    }
}
