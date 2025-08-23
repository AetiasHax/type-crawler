use std::fmt::Display;

use crate::{
    EnumDecl, Env, StructDecl, Typedef, Types, UnionDecl,
    error::{AlignofSnafu, ParseError, SizeofSnafu, UnsupportedEntitySnafu, UnsupportedTypeSnafu},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    USize {
        size: usize,
    },
    SSize {
        size: usize,
    },
    U64,
    U32,
    U16,
    U8,
    S64,
    S32,
    S16,
    S8,
    F32,
    F64,
    LongDouble {
        size: usize,
        alignment: usize,
    },
    Char16,
    Char32,
    WChar {
        size: usize,
    },
    Bool,
    Void,
    Reference {
        size: usize,
        referenced_type: Box<TypeKind>,
    },
    Pointer {
        size: usize,
        pointee_type: Box<TypeKind>,
    },
    MemberPointer {
        size: usize,
        pointee_type: Box<TypeKind>,
        record_name: String,
    },
    Array {
        element_type: Box<TypeKind>,
        size: Option<usize>, // None for incomplete arrays
    },
    Function {
        return_type: Box<TypeKind>,
        parameters: Vec<TypeKind>,
    },
    Struct(StructDecl),
    Class(StructDecl),
    Union(UnionDecl),
    Enum(EnumDecl),
    Typedef(Box<Typedef>),
    Named(String),
}

impl TypeKind {
    pub fn new(env: &Env, types: &Types, ty: clang::Type) -> Result<Self, ParseError> {
        let kind = ty.get_kind();
        match kind {
            clang::TypeKind::ULong => Ok(TypeKind::USize { size: env.word_size().bytes() }),
            clang::TypeKind::Long => Ok(TypeKind::SSize { size: env.word_size().bytes() }),
            clang::TypeKind::ULongLong => Ok(TypeKind::U64),
            clang::TypeKind::UInt => Ok(TypeKind::U32),
            clang::TypeKind::UShort => Ok(TypeKind::U16),
            clang::TypeKind::UChar => Ok(TypeKind::U8),
            clang::TypeKind::LongLong => Ok(TypeKind::S64),
            clang::TypeKind::Int => Ok(TypeKind::S32),
            clang::TypeKind::Short => Ok(TypeKind::S16),
            clang::TypeKind::CharS => Ok(TypeKind::S8),
            clang::TypeKind::CharU => Ok(TypeKind::U8),
            clang::TypeKind::Float => Ok(TypeKind::F32),
            clang::TypeKind::Double => Ok(TypeKind::F64),
            clang::TypeKind::LongDouble => Ok(TypeKind::LongDouble {
                size: ty.get_sizeof().map_err(|e| {
                    SizeofSnafu { type_name: ty.get_display_name(), error: e }.build()
                })?,
                alignment: ty.get_alignof().map_err(|e| {
                    AlignofSnafu { type_name: ty.get_display_name(), error: e }.build()
                })?,
            }),
            clang::TypeKind::Char16 => Ok(TypeKind::Char16),
            clang::TypeKind::Char32 => Ok(TypeKind::Char32),
            clang::TypeKind::WChar => Ok(TypeKind::WChar {
                size: ty.get_sizeof().map_err(|e| {
                    SizeofSnafu { type_name: ty.get_display_name(), error: e }.build()
                })?,
            }),
            clang::TypeKind::Bool => Ok(TypeKind::Bool),
            clang::TypeKind::Void => Ok(TypeKind::Void),
            clang::TypeKind::LValueReference | clang::TypeKind::Pointer => {
                let pointee_type = ty.get_pointee_type().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("Pointer type without pointee type: {ty:?}"),
                    }
                    .build()
                })?;
                let inner_type = TypeKind::new(env, types, pointee_type)?;
                let size = ty.get_sizeof().map_err(|e| {
                    SizeofSnafu { type_name: ty.get_display_name(), error: e }.build()
                })?;
                let pointee_type = Box::new(inner_type);

                if kind == clang::TypeKind::LValueReference {
                    Ok(TypeKind::Reference { size, referenced_type: pointee_type })
                } else {
                    Ok(TypeKind::Pointer { size, pointee_type })
                }
            }
            clang::TypeKind::MemberPointer => {
                let pointee_type = ty.get_pointee_type().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("MemberPointer type without pointee type: {ty:?}"),
                    }
                    .build()
                })?;
                let inner_type = TypeKind::new(env, types, pointee_type)?;
                let size = ty.get_sizeof().map_err(|e| {
                    SizeofSnafu { type_name: ty.get_display_name(), error: e }.build()
                })?;
                let pointee_type = Box::new(inner_type);

                let record_name = ty
                    .get_class_type()
                    .ok_or_else(|| {
                        UnsupportedTypeSnafu {
                            message: format!("MemberPointer type without class type: {ty:?}"),
                        }
                        .build()
                    })?
                    .get_display_name();

                Ok(TypeKind::MemberPointer { size, pointee_type, record_name })
            }
            clang::TypeKind::IncompleteArray => {
                let element_type = ty.get_element_type().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("IncompleteArray type without element type: {ty:?}"),
                    }
                    .build()
                })?;
                let inner_type = TypeKind::new(env, types, element_type)?;
                Ok(TypeKind::Array { element_type: Box::new(inner_type), size: None })
            }
            clang::TypeKind::ConstantArray => {
                let element_type = ty.get_element_type().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("ConstantArray type without element type: {ty:?}"),
                    }
                    .build()
                })?;
                let size = ty.get_size().ok_or_else(|| {
                    UnsupportedTypeSnafu { message: format!("ConstantArray without size: {ty:?}") }
                        .build()
                })?;
                let inner_type = TypeKind::new(env, types, element_type)?;
                Ok(TypeKind::Array { element_type: Box::new(inner_type), size: Some(size) })
            }
            clang::TypeKind::FunctionPrototype => {
                let return_type = ty.get_result_type().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("FunctionPrototype without return type: {ty:?}"),
                    }
                    .build()
                })?;
                let parameters = ty.get_argument_types().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("FunctionPrototype without parameters: {ty:?}"),
                    }
                    .build()
                })?;
                let return_type = TypeKind::new(env, types, return_type)?;
                let parameters = parameters
                    .into_iter()
                    .map(|param| TypeKind::new(env, types, param))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeKind::Function { return_type: Box::new(return_type), parameters })
            }
            clang::TypeKind::Elaborated => {
                let elaborated_type = ty.get_elaborated_type().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("Elaborated type without type: {ty:?}"),
                    }
                    .build()
                })?;
                let elaborated_decl = elaborated_type.get_declaration().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("Elaborated type without declaration: {ty:?}"),
                    }
                    .build()
                })?;
                if elaborated_decl.is_anonymous() {
                    TypeKind::new(env, types, elaborated_type)
                } else {
                    let name = elaborated_decl.get_name().ok_or_else(|| {
                        UnsupportedTypeSnafu {
                            message: format!("Elaborated type declaration without name: {ty:?}"),
                        }
                        .build()
                    })?;
                    match name.as_str() {
                        "bool" => Ok(TypeKind::Bool), // "bool" not defined in C
                        _ => Ok(TypeKind::Named(name)),
                    }
                }
            }
            clang::TypeKind::Record => {
                let node = ty.get_declaration().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("Record type without declaration: {ty:?}"),
                    }
                    .build()
                })?;
                match node.get_kind() {
                    clang::EntityKind::StructDecl => {
                        let struct_decl = StructDecl::new(env, types, None, ty)?;
                        Ok(TypeKind::Struct(struct_decl))
                    }
                    clang::EntityKind::ClassDecl => {
                        let struct_decl = StructDecl::new(env, types, None, ty)?;
                        Ok(TypeKind::Class(struct_decl))
                    }
                    clang::EntityKind::UnionDecl => {
                        let union_decl = UnionDecl::new(env, types, None, ty)?;
                        Ok(TypeKind::Union(union_decl))
                    }
                    _ => UnsupportedEntitySnafu {
                        at: "struct/union".to_string(),
                        message: format!(
                            "Unsupported entity kind in record: {:?}",
                            node.get_kind()
                        ),
                    }
                    .fail(),
                }
            }
            clang::TypeKind::Enum => {
                let decl = ty.get_declaration().ok_or_else(|| {
                    UnsupportedTypeSnafu {
                        message: format!("Enum type without declaration: {ty:?}"),
                    }
                    .build()
                })?;
                let name = decl.get_name();
                Ok(TypeKind::Enum(EnumDecl::new(name, &decl)?))
            }
            _ => {
                panic!("Unsupported type: {:?} for name: {}", ty.get_kind(), ty.get_display_name())
            }
        }
    }

    pub fn size(&self, types: &Types) -> usize {
        match self {
            TypeKind::USize { size } | TypeKind::SSize { size } => *size,
            TypeKind::U64 | TypeKind::S64 => 8,
            TypeKind::U32 | TypeKind::S32 => 4,
            TypeKind::U16 | TypeKind::S16 => 2,
            TypeKind::U8 | TypeKind::S8 => 1,
            TypeKind::F32 => 4,
            TypeKind::F64 => 8,
            TypeKind::LongDouble { size, .. } => *size,
            TypeKind::Char16 => 2,
            TypeKind::Char32 => 4,
            TypeKind::WChar { size } => *size,
            TypeKind::Bool => 1,
            TypeKind::Void => 0,
            TypeKind::Reference { size, .. } => *size,
            TypeKind::Pointer { size, .. } => *size,
            TypeKind::MemberPointer { size, .. } => *size,
            TypeKind::Array { element_type, size } => {
                if let Some(size) = size {
                    let stride =
                        element_type.size(types).next_multiple_of(element_type.alignment(types));
                    size * stride
                } else {
                    0
                }
            }
            TypeKind::Function { .. } => 0,
            TypeKind::Struct(struct_decl) => struct_decl.size(),
            TypeKind::Class(class_decl) => class_decl.size(),
            TypeKind::Union(union_decl) => union_decl.size(),
            TypeKind::Enum(enum_decl) => enum_decl.size(),
            TypeKind::Typedef(typedef) => typedef.underlying_type().size(types),
            TypeKind::Named(name) => types.get(name).map(|ty| ty.size(types)).unwrap_or(0),
        }
    }

    pub fn alignment(&self, types: &Types) -> usize {
        match self {
            TypeKind::USize { size } | TypeKind::SSize { size } => *size,
            TypeKind::U64 | TypeKind::S64 => 8,
            TypeKind::U32 | TypeKind::S32 => 4,
            TypeKind::U16 | TypeKind::S16 => 2,
            TypeKind::U8 | TypeKind::S8 => 1,
            TypeKind::F32 => 4,
            TypeKind::F64 => 8,
            TypeKind::LongDouble { alignment, .. } => *alignment,
            TypeKind::Char16 => 2,
            TypeKind::Char32 => 4,
            TypeKind::WChar { size } => *size,
            TypeKind::Bool => 1,
            TypeKind::Void => 0,
            TypeKind::Reference { size, .. } => *size,
            TypeKind::Pointer { size, .. } => *size,
            TypeKind::MemberPointer { size, .. } => *size,
            TypeKind::Array { element_type, .. } => element_type.alignment(types),
            TypeKind::Function { .. } => 0,
            TypeKind::Struct(struct_decl) => struct_decl.alignment(),
            TypeKind::Class(class_decl) => class_decl.alignment(),
            TypeKind::Union(union_decl) => union_decl.alignment(),
            TypeKind::Enum(enum_decl) => enum_decl.alignment(),
            TypeKind::Typedef(typedef) => typedef.underlying_type().alignment(types),
            TypeKind::Named(name) => types.get(name).map(|ty| ty.alignment(types)).unwrap_or(0),
        }
    }

    pub fn stride(&self, types: &Types) -> usize {
        let size = self.size(types);
        let alignment = self.alignment(types);
        size.next_multiple_of(alignment)
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            TypeKind::Struct(struct_decl) => struct_decl.name(),
            TypeKind::Class(class_decl) => class_decl.name(),
            TypeKind::Union(union_decl) => union_decl.name(),
            TypeKind::Enum(enum_decl) => enum_decl.name(),
            TypeKind::Typedef(typedef) => Some(typedef.name()),
            TypeKind::Named(name) => Some(name),
            _ => None,
        }
    }

    pub fn expand_named<'a>(&'a self, types: &'a Types) -> Option<&'a TypeKind> {
        match self {
            TypeKind::Named(name) => types.get(name),
            _ => Some(self),
        }
    }

    pub fn is_forward_decl(&self) -> bool {
        match self {
            TypeKind::Struct(struct_decl) => struct_decl.is_forward_decl(),
            TypeKind::Class(class_decl) => class_decl.is_forward_decl(),
            _ => false,
        }
    }

    pub fn as_struct<'a>(&'a self, types: &'a Types) -> Option<&'a StructDecl> {
        match self {
            TypeKind::Struct(struct_decl) => Some(struct_decl),
            TypeKind::Class(class_decl) => Some(class_decl),
            TypeKind::Named(name) => types.get(name)?.as_struct(types),
            _ => None,
        }
    }
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::USize { size } => write!(f, "usize({size})"),
            TypeKind::SSize { size } => write!(f, "ssize({size})"),
            TypeKind::U64 => write!(f, "u64"),
            TypeKind::U32 => write!(f, "u32"),
            TypeKind::U16 => write!(f, "u16"),
            TypeKind::U8 => write!(f, "u8"),
            TypeKind::S64 => write!(f, "s64"),
            TypeKind::S32 => write!(f, "s32"),
            TypeKind::S16 => write!(f, "s16"),
            TypeKind::S8 => write!(f, "s8"),
            TypeKind::F32 => write!(f, "f32"),
            TypeKind::F64 => write!(f, "f64"),
            TypeKind::LongDouble { size, .. } => write!(f, "long double({size})"),
            TypeKind::Char16 => write!(f, "char16"),
            TypeKind::Char32 => write!(f, "char32"),
            TypeKind::WChar { size } => write!(f, "wchar({size})"),
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Void => write!(f, "void"),
            TypeKind::Reference { referenced_type, .. } => {
                write!(f, "{}&", referenced_type)
            }
            TypeKind::Pointer { pointee_type, .. } => {
                write!(f, "{}*", pointee_type)
            }
            TypeKind::MemberPointer { pointee_type, record_name, .. } => {
                write!(f, "{} {}::*", pointee_type, record_name)
            }
            TypeKind::Array { element_type, size } => {
                if let Some(size) = size {
                    write!(f, "{}[{}]", element_type, size)
                } else {
                    write!(f, "{}[]", element_type)
                }
            }
            TypeKind::Function { return_type, parameters } => {
                let params =
                    parameters.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "{return_type} function({params})")
            }
            TypeKind::Struct(struct_decl) => write!(f, "struct {struct_decl}"),
            TypeKind::Class(class_decl) => write!(f, "class {class_decl}"),
            TypeKind::Union(union_decl) => write!(f, "union {union_decl}"),
            TypeKind::Enum(enum_decl) => write!(f, "enum {enum_decl}"),
            TypeKind::Typedef(typedef) => write!(f, "{typedef}"),
            TypeKind::Named(name) => write!(f, "{name}"),
        }
    }
}
