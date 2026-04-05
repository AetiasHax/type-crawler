mod enum_decl;
mod field;
mod path;
mod struct_decl;
mod template_class;
mod type_kind;
mod typedef;
mod union_decl;

pub use enum_decl::{EnumConstant, EnumDecl};
use exn::bail;
pub use field::Field;
use indexmap::IndexMap;
pub use path::TypePath;
pub use struct_decl::{StructDecl, StructField};
pub use template_class::TemplateClass;
pub use type_kind::TypeKind;
pub use typedef::Typedef;
pub use union_decl::UnionDecl;

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Types {
    types: IndexMap<TypePath, TypeKind>,
    template_classes: IndexMap<TypePath, TemplateClass>,
}

#[derive(Debug, derive_more::Display)]
pub enum ExtendTypesError {
    #[display("Type with the same name but different definitions:\n{left}\nand\n{right}")]
    ConflictingTypes { left: Box<TypeKind>, right: Box<TypeKind> },
    #[display("Template class with the same name but different definitions:\n{left}\nand\n{right}")]
    ConflictingTemplateClasses { left: Box<TemplateClass>, right: Box<TemplateClass> },
}
impl std::error::Error for ExtendTypesError {}

impl Types {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_type(&mut self, kind: TypeKind) -> exn::Result<bool, ExtendTypesError> {
        if let TypeKind::Typedef(typedef) = &kind
            && let TypeKind::Named(path) = typedef.underlying_type()
            && typedef.path() == path
        {
            // Avoid adding a typedef that has the same path as its underlying type
            // Example: typedef struct MyStruct {...} MyStruct;
            return Ok(false);
        };
        if let Some(path) = kind.path().cloned() {
            match self.types.entry(path) {
                indexmap::map::Entry::Occupied(mut entry) => {
                    let current = entry.get();
                    if current.is_forward_decl() {
                        entry.insert(kind);
                    } else if !kind.is_forward_decl() && current != &kind {
                        bail!(ExtendTypesError::ConflictingTypes {
                            left: Box::new(current.clone()),
                            right: Box::new(kind),
                        });
                    }
                }
                indexmap::map::Entry::Vacant(entry) => {
                    entry.insert(kind);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn types(&self) -> impl Iterator<Item = &TypeKind> {
        self.types.values()
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, path: impl Into<TypePath>) -> Option<&TypeKind> {
        self.types.get(&path.into())
    }

    pub fn extend(&mut self, other: Types) -> exn::Result<(), ExtendTypesError> {
        for (name, value) in other.types {
            match self.types.entry(name.clone()) {
                indexmap::map::Entry::Occupied(mut entry) => {
                    let current = entry.get();
                    if current.is_forward_decl() {
                        entry.insert(value);
                    } else if !value.is_forward_decl() && current != &value {
                        bail!(ExtendTypesError::ConflictingTypes {
                            left: Box::new(current.clone()),
                            right: Box::new(value),
                        });
                    }
                }
                indexmap::map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }
        Ok(())
    }

    pub fn add_template_class(
        &mut self,
        template_class: TemplateClass,
    ) -> exn::Result<bool, ExtendTypesError> {
        let path = template_class.path();
        match self.template_classes.entry(path.clone()) {
            indexmap::map::Entry::Occupied(entry) => {
                let current = entry.get();
                if current != &template_class {
                    bail!(ExtendTypesError::ConflictingTemplateClasses {
                        left: Box::new(current.clone()),
                        right: Box::new(template_class),
                    });
                }
            }
            indexmap::map::Entry::Vacant(entry) => {
                entry.insert(template_class);
            }
        }
        Ok(true)
    }

    pub fn template_classes(&self) -> impl Iterator<Item = &TemplateClass> {
        self.template_classes.values()
    }
}
