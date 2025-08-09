use std::fmt::Display;

use crate::TypeKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Option<String>,
    pub kind: TypeKind,
}

impl Field {
    pub fn new(name: Option<String>, kind: TypeKind) -> Self {
        Self { name, kind }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.as_deref().unwrap_or("<anon>");
        write!(f, "{}: {:?}", name, self.kind)
    }
}
