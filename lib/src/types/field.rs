use std::fmt::Display;

use crate::TypeKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: String,
    kind: TypeKind,
}

impl Field {
    pub fn new(name: String, kind: TypeKind) -> Self {
        Self { name, kind }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.kind)
    }
}
