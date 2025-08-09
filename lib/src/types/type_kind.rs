use crate::{Env, StructDecl, Types, UnionDecl, error::ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    USize,
    SSize,
    U64,
    U32,
    U16,
    U8,
    S64,
    S32,
    S16,
    S8,
    Bool,
    Void,
    Pointer(Box<TypeKind>),
    Array {
        element_type: Box<TypeKind>,
        size: Option<usize>, // None for incomplete arrays
    },
    Function {
        return_type: Box<TypeKind>,
        parameters: Vec<TypeKind>,
    },
    Struct(StructDecl),
    Union(UnionDecl),
    Named(String),
}

impl TypeKind {
    pub fn new(ty: clang::Type) -> Result<Self, ParseError> {
        match ty.get_kind() {
            clang::TypeKind::ULong => Ok(TypeKind::USize),
            clang::TypeKind::Long => Ok(TypeKind::SSize),
            clang::TypeKind::ULongLong => Ok(TypeKind::U64),
            clang::TypeKind::UInt => Ok(TypeKind::U32),
            clang::TypeKind::UShort => Ok(TypeKind::U16),
            clang::TypeKind::UChar => Ok(TypeKind::U8),
            clang::TypeKind::LongLong => Ok(TypeKind::S64),
            clang::TypeKind::Int => Ok(TypeKind::S32),
            clang::TypeKind::Short => Ok(TypeKind::S16),
            clang::TypeKind::CharS => Ok(TypeKind::S8),
            clang::TypeKind::Bool => Ok(TypeKind::Bool),
            clang::TypeKind::Void => Ok(TypeKind::Void),
            clang::TypeKind::Pointer => {
                let pointee_type = ty.get_pointee_type().ok_or_else(|| {
                    ParseError::UnsupportedType(format!(
                        "Pointer type without pointee type: {ty:?}"
                    ))
                })?;
                let inner_type = TypeKind::new(pointee_type)?;
                Ok(TypeKind::Pointer(Box::new(inner_type)))
            }
            clang::TypeKind::IncompleteArray => {
                let element_type = ty.get_element_type().ok_or_else(|| {
                    ParseError::UnsupportedType(format!(
                        "IncompleteArray type without element type: {ty:?}"
                    ))
                })?;
                let inner_type = TypeKind::new(element_type)?;
                Ok(TypeKind::Array { element_type: Box::new(inner_type), size: None })
            }
            clang::TypeKind::ConstantArray => {
                let element_type = ty.get_element_type().ok_or_else(|| {
                    ParseError::UnsupportedType(format!(
                        "ConstantArray type without element type: {ty:?}"
                    ))
                })?;
                let size = ty.get_size().ok_or_else(|| {
                    ParseError::UnsupportedType(format!("ConstantArray without size: {ty:?}"))
                })?;
                let inner_type = TypeKind::new(element_type)?;
                Ok(TypeKind::Array { element_type: Box::new(inner_type), size: Some(size) })
            }
            clang::TypeKind::FunctionPrototype => {
                let return_type = ty.get_result_type().ok_or_else(|| {
                    ParseError::UnsupportedType(format!(
                        "FunctionPrototype without return type: {ty:?}"
                    ))
                })?;
                let parameters = ty.get_argument_types().ok_or_else(|| {
                    ParseError::UnsupportedType(format!(
                        "FunctionPrototype without parameters: {ty:?}"
                    ))
                })?;
                let return_type = TypeKind::new(return_type)?;
                let parameters =
                    parameters.into_iter().map(TypeKind::new).collect::<Result<Vec<_>, _>>()?;
                Ok(TypeKind::Function { return_type: Box::new(return_type), parameters })
            }
            clang::TypeKind::Elaborated => {
                let elaborated_type = ty.get_elaborated_type().ok_or_else(|| {
                    ParseError::UnsupportedType(format!("Elaborated type without name: {ty:?}"))
                })?;
                let name = elaborated_type.get_display_name();
                match name.as_str() {
                    "bool" => Ok(TypeKind::Bool), // "bool" not defined in C
                    _ => Ok(TypeKind::Named(name)),
                }
            }
            _ => {
                panic!("Unsupported type: {:?} for name: {}", ty.get_kind(), ty.get_display_name())
            }
        }
    }

    pub fn size(&self, env: &Env, types: &Types) -> Option<usize> {
        Some(match self {
            TypeKind::USize | TypeKind::SSize => env.word_size().bits() / 8,
            TypeKind::U64 | TypeKind::S64 => 8,
            TypeKind::U32 | TypeKind::S32 => 4,
            TypeKind::U16 | TypeKind::S16 => 2,
            TypeKind::U8 | TypeKind::S8 => 1,
            TypeKind::Bool => 1,
            TypeKind::Void => return None,
            TypeKind::Pointer(_) => env.word_size().bits() / 8,
            TypeKind::Array { element_type, size } => {
                if let Some(size) = size {
                    let stride = element_type
                        .size(env, types)?
                        .next_multiple_of(element_type.alignment(env, types)?);
                    size * stride
                } else {
                    return None;
                }
            }
            TypeKind::Function { .. } => return None,
            TypeKind::Struct(struct_decl) => struct_decl.size(env, types)?,
            TypeKind::Union(union_decl) => union_decl.size(env, types)?,
            TypeKind::Named(name) => types.get(name)?.size(env, types)?,
        })
    }

    pub fn alignment(&self, env: &Env, types: &Types) -> Option<usize> {
        Some(match self {
            TypeKind::USize | TypeKind::SSize => env.word_size().bits() / 8,
            TypeKind::U64 | TypeKind::S64 => 8,
            TypeKind::U32 | TypeKind::S32 => 4,
            TypeKind::U16 | TypeKind::S16 => 2,
            TypeKind::U8 | TypeKind::S8 => 1,
            TypeKind::Bool => 1,
            TypeKind::Void => return None,
            TypeKind::Pointer(_) => env.word_size().bits() / 8,
            TypeKind::Array { element_type, .. } => element_type.alignment(env, types)?,
            TypeKind::Function { .. } => return None,
            TypeKind::Struct(struct_decl) => struct_decl.alignment(env, types)?,
            TypeKind::Union(union_decl) => union_decl.alignment(env, types)?,
            TypeKind::Named(name) => types.get(name)?.alignment(env, types)?,
        })
    }
}
