use std::{borrow::Cow, fmt::Display};

use crate::error::{ParseError, UnsupportedTypeSnafu};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypePath {
    namespaces: Vec<String>,
    name: String,
}

impl TypePath {
    pub fn new(namespaces: Vec<String>, name: impl Into<String>) -> Self {
        Self { namespaces, name: name.into() }
    }

    pub fn global(name: impl Into<String>) -> Self {
        Self { name: name.into(), namespaces: Vec::new() }
    }

    pub fn from_entity(entity: &clang::Entity) -> Result<Self, ParseError> {
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

        let name = entity.get_name().ok_or_else(|| {
            UnsupportedTypeSnafu {
                message: format!("Elaborated type declaration without name: {entity:?}"),
            }
            .build()
        })?;

        Ok(Self { namespaces, name })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn namespaces(&self) -> &[String] {
        &self.namespaces
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
        Ok(())
    }
}

impl<'a> From<&'a TypePath> for Cow<'a, TypePath> {
    fn from(val: &'a TypePath) -> Self {
        Cow::Borrowed(val)
    }
}
