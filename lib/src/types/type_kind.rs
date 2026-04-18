use std::fmt::Display;

use crate::{
    EnumDecl, Env, StructDecl, TypePath, Typedef, Types, UnionDecl,
    error::{ExnExt as _, OptionExt as _, ResultExt as _, bail_str, error_type},
};

error_type!(TypeKindError);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    Named(TypePath),
    TemplateParam(String),
    TemplateClassSpec(StructDecl),
}

impl TypeKind {
    pub fn new(env: &Env, types: &Types, ty: clang::Type) -> exn::Result<Self, TypeKindError> {
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
            clang::TypeKind::SChar => Ok(TypeKind::S8),
            clang::TypeKind::CharS => Ok(TypeKind::S8),
            clang::TypeKind::CharU => Ok(TypeKind::U8),
            clang::TypeKind::Float => Ok(TypeKind::F32),
            clang::TypeKind::Double => Ok(TypeKind::F64),
            clang::TypeKind::LongDouble => Ok(TypeKind::LongDouble {
                size: ty.get_sizeof().or_raise_str(|| {
                    format!("Failed to get size of long double type '{}'", ty.get_display_name())
                })?,
                alignment: ty.get_alignof().or_raise_str(|| {
                    format!("Failed to get alignment of {}", ty.get_display_name())
                })?,
            }),
            clang::TypeKind::Char16 => Ok(TypeKind::Char16),
            clang::TypeKind::Char32 => Ok(TypeKind::Char32),
            clang::TypeKind::WChar => Ok(TypeKind::WChar {
                size: ty.get_sizeof().or_raise_str(|| {
                    format!("Failed to get size of wide char type '{}'", ty.get_display_name())
                })?,
            }),
            clang::TypeKind::Bool => Ok(TypeKind::Bool),
            clang::TypeKind::Void => Ok(TypeKind::Void),
            clang::TypeKind::LValueReference | clang::TypeKind::Pointer => {
                let pointee_type = ty
                    .get_pointee_type()
                    .ok_or_raise_str(|| format!("Pointer type without pointee type: {:?}", ty))?;
                let inner_type = TypeKind::new(env, types, pointee_type)?;
                let size = env.word_size().bytes();
                let pointee_type = Box::new(inner_type);

                if kind == clang::TypeKind::LValueReference {
                    Ok(TypeKind::Reference { size, referenced_type: pointee_type })
                } else {
                    Ok(TypeKind::Pointer { size, pointee_type })
                }
            }
            clang::TypeKind::MemberPointer => {
                let pointee_type = ty.get_pointee_type().ok_or_raise_str(|| {
                    format!("MemberPointer type without pointee type: {:?}", ty)
                })?;
                let inner_type = TypeKind::new(env, types, pointee_type)?;
                let size = ty.get_sizeof().or_raise_str(|| {
                    format!("Failed to get size of member pointer type '{}'", ty.get_display_name())
                })?;
                let pointee_type = Box::new(inner_type);

                let record_name = ty
                    .get_class_type()
                    .ok_or_raise_str(|| format!("MemberPointer type without class type: {:?}", ty))?
                    .get_display_name();

                Ok(TypeKind::MemberPointer { size, pointee_type, record_name })
            }
            clang::TypeKind::IncompleteArray => {
                let element_type = ty.get_element_type().ok_or_raise_str(|| {
                    format!("IncompleteArray type without element type: {:?}", ty)
                })?;
                let inner_type = TypeKind::new(env, types, element_type)?;
                Ok(TypeKind::Array { element_type: Box::new(inner_type), size: None })
            }
            clang::TypeKind::ConstantArray => {
                let element_type = ty.get_element_type().ok_or_raise_str(|| {
                    format!("ConstantArray type without element type: {:?}", ty)
                })?;
                let size = ty
                    .get_size()
                    .ok_or_raise_str(|| format!("ConstantArray without size: {:?}", ty))?;
                let inner_type = TypeKind::new(env, types, element_type)?;
                Ok(TypeKind::Array { element_type: Box::new(inner_type), size: Some(size) })
            }
            clang::TypeKind::FunctionPrototype => {
                let return_type = ty.get_result_type().ok_or_raise_str(|| {
                    format!("FunctionPrototype without return type: {:?}", ty)
                })?;
                let parameters = ty.get_argument_types().ok_or_raise_str(|| {
                    format!("FunctionPrototype without parameters: {:?}", ty)
                })?;
                let return_type = TypeKind::new(env, types, return_type)?;
                let parameters = parameters
                    .into_iter()
                    .map(|param| TypeKind::new(env, types, param))
                    .collect::<exn::Result<Vec<_>, _>>()?;
                Ok(TypeKind::Function { return_type: Box::new(return_type), parameters })
            }
            clang::TypeKind::Elaborated => {
                let elaborated_type = ty
                    .get_elaborated_type()
                    .ok_or_raise_str(|| format!("Elaborated type without type: {:?}", ty))?;
                let elaborated_decl = elaborated_type
                    .get_declaration()
                    .ok_or_raise_str(|| format!("Elaborated type without declaration: {:?}", ty))?;
                if elaborated_decl.is_anonymous() {
                    TypeKind::new(env, types, elaborated_type)
                } else {
                    let path = TypePath::from_entity(&elaborated_decl).or_raise_str(|| {
                        format!(
                            "Failed to get path to elaborated type {}",
                            elaborated_type.get_display_name()
                        )
                    })?;
                    if let Some(template_args) = ty.get_template_argument_types() {
                        let args = template_args
                            .iter()
                            .enumerate()
                            .map(|(i, arg)| {
                                let Some(arg) = arg else {
                                    bail_str!(
                                        "Template argument at index {} is None for template class {}",
                                        i,
                                        path
                                    );
                                };
                                let kind =TypeKind::new(env, types, *arg).or_raise_str(|| format!("Failed to derive type for template argument at index {} for template class {}", i, path))?;
                                Ok(kind)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let template_class = types
                            .get_template_class(path.clone())
                            .ok_or_raise_str(|| format!("Template class not found: {}", path))?;
                        let specialized = template_class.specialize(env, types, &args).or_raise_str(|| format!("Failed to specialize template class {} with template arguments {:?}", path, args))?;
                        Ok(TypeKind::TemplateClassSpec(specialized))
                    } else if path == TypePath::global("bool") {
                        Ok(TypeKind::Bool) // "bool" not defined in C
                    } else {
                        Ok(TypeKind::Named(path))
                    }
                }
            }
            clang::TypeKind::Record => {
                let node = ty
                    .get_declaration()
                    .ok_or_raise_str(|| format!("Record type without declaration: {:?}", ty))?;
                match node.get_kind() {
                    clang::EntityKind::StructDecl => {
                        let struct_decl =
                            StructDecl::new(env, types, None, &node).or_raise_str(|| {
                                format!(
                                    "Failed to parse struct AST for record type {}",
                                    ty.get_display_name()
                                )
                            })?;
                        Ok(TypeKind::Struct(struct_decl))
                    }
                    clang::EntityKind::ClassDecl => {
                        let struct_decl =
                            StructDecl::new(env, types, None, &node).or_raise_str(|| {
                                format!(
                                    "Failed to parse class AST for record type {}",
                                    ty.get_display_name()
                                )
                            })?;
                        Ok(TypeKind::Class(struct_decl))
                    }
                    clang::EntityKind::UnionDecl => {
                        let union_decl =
                            UnionDecl::new(env, types, None, ty).or_raise_str(|| {
                                format!(
                                    "Failed to parse union AST for record type {}",
                                    ty.get_display_name()
                                )
                            })?;
                        Ok(TypeKind::Union(union_decl))
                    }
                    _ => {
                        bail_str!("Unsupported entity in struct/union: {:?}", node.get_kind());
                    }
                }
            }
            clang::TypeKind::Enum => {
                let decl = ty
                    .get_declaration()
                    .ok_or_raise_str(|| format!("Enum type without declaration: {:?}", ty))?;
                let path = TypePath::from_entity(&decl).or_raise_str(|| {
                    format!("Failed to get path to enum type '{}'", ty.get_display_name())
                })?;
                Ok(TypeKind::Enum(EnumDecl::new(Some(path), &decl).or_raise_str(|| {
                    format!("Failed to parse AST for enum '{}'", ty.get_display_name())
                })?))
            }
            clang::TypeKind::Unexposed => {
                let name = ty.get_display_name();
                Ok(TypeKind::TemplateParam(name))
            }
            clang::TypeKind::Typedef => {
                let decl = ty
                    .get_declaration()
                    .ok_or_raise_str(|| format!("Typedef type without declaration: {:?}", ty))?;
                let path = TypePath::from_entity(&decl).or_raise_str(|| {
                    format!("Failed to get path to typedef type '{}'", ty.get_display_name())
                })?;
                Ok(TypeKind::Named(path))
            }
            _ => {
                bail_str!(
                    "Unsupported type: {:?} for name: {}",
                    ty.get_kind(),
                    ty.get_display_name()
                );
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
            TypeKind::Named(name) => types.get(name.clone()).map(|ty| ty.size(types)).unwrap_or(0),
            TypeKind::TemplateParam(_) => 0,
            TypeKind::TemplateClassSpec(specialized) => specialized.size(),
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
            TypeKind::Named(name) => {
                types.get(name.clone()).map(|ty| ty.alignment(types)).unwrap_or(0)
            }
            TypeKind::TemplateParam(_) => 0,
            TypeKind::TemplateClassSpec(specialized) => specialized.alignment(),
        }
    }

    pub fn stride(&self, types: &Types) -> usize {
        let size = self.size(types);
        let alignment = self.alignment(types);
        size.next_multiple_of(alignment)
    }

    #[deprecated(note = "use path().map(|p| p.name()) instead")]
    #[allow(deprecated)]
    pub fn name(&self) -> Option<&str> {
        match self {
            TypeKind::Struct(struct_decl) => struct_decl.name(),
            TypeKind::Class(class_decl) => class_decl.name(),
            TypeKind::Union(union_decl) => union_decl.name(),
            TypeKind::Enum(enum_decl) => enum_decl.name(),
            TypeKind::Typedef(typedef) => Some(typedef.name()),
            TypeKind::Named(path) => Some(path.name()),
            _ => None,
        }
    }

    pub fn path(&self) -> Option<&TypePath> {
        match self {
            TypeKind::Struct(struct_decl) => struct_decl.path(),
            TypeKind::Class(class_decl) => class_decl.path(),
            TypeKind::Union(union_decl) => union_decl.path(),
            TypeKind::Enum(enum_decl) => enum_decl.path(),
            TypeKind::Typedef(typedef) => Some(typedef.path()),
            TypeKind::Named(path) => Some(path),
            _ => None,
        }
    }

    pub fn expand_named<'a>(&'a self, types: &'a Types) -> Option<&'a TypeKind> {
        match self {
            TypeKind::Named(name) => types.get(name.clone()),
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
            TypeKind::Named(name) => types.get(name.clone())?.as_struct(types),
            _ => None,
        }
    }

    pub fn is_virtual(&self, types: &Types) -> bool {
        match self {
            TypeKind::Struct(struct_decl) => struct_decl.is_virtual(),
            TypeKind::Class(class_decl) => class_decl.is_virtual(),
            TypeKind::Named(type_path) => {
                types.get(type_path.clone()).map(|t| t.is_virtual(types)).unwrap_or(false)
            }
            _ => false,
        }
    }

    pub fn replace_template_parameters<Cb>(
        &self,
        types: &Types,
        get_param_type: Cb,
    ) -> exn::Result<TypeKind, TypeKindError>
    where
        Cb: Fn(&str) -> Option<TypeKind> + Copy,
    {
        match self {
            TypeKind::USize { .. }
            | TypeKind::SSize { .. }
            | TypeKind::U64
            | TypeKind::U32
            | TypeKind::U16
            | TypeKind::U8
            | TypeKind::S64
            | TypeKind::S32
            | TypeKind::S16
            | TypeKind::S8
            | TypeKind::F32
            | TypeKind::F64
            | TypeKind::LongDouble { .. }
            | TypeKind::Char16
            | TypeKind::Char32
            | TypeKind::WChar { .. }
            | TypeKind::Bool
            | TypeKind::Void => Ok(self.clone()),
            TypeKind::Reference { size, referenced_type } => Ok(TypeKind::Reference {
                size: *size,
                referenced_type: Box::new(
                    referenced_type.replace_template_parameters(types, get_param_type)?,
                ),
            }),
            TypeKind::Pointer { size, pointee_type } => Ok(TypeKind::Pointer {
                size: *size,
                pointee_type: Box::new(
                    pointee_type.replace_template_parameters(types, get_param_type)?,
                ),
            }),
            TypeKind::MemberPointer { size, pointee_type, record_name } => {
                Ok(TypeKind::MemberPointer {
                    size: *size,
                    pointee_type: Box::new(
                        pointee_type.replace_template_parameters(types, get_param_type)?,
                    ),
                    record_name: record_name.clone(),
                })
            }
            TypeKind::Array { element_type, size } => Ok(TypeKind::Array {
                element_type: Box::new(
                    element_type.replace_template_parameters(types, get_param_type)?,
                ),
                size: *size,
            }),
            TypeKind::Function { return_type, parameters } => Ok(TypeKind::Function {
                return_type: Box::new(
                    return_type.replace_template_parameters(types, get_param_type)?,
                ),
                parameters: parameters
                    .iter()
                    .map(|param| param.replace_template_parameters(types, get_param_type))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            TypeKind::Struct(struct_decl) => Ok(TypeKind::Struct(
                struct_decl
                    .replace_template_parameters(types, get_param_type)
                    .or_raise_str(|| "Failed to specialize struct inside template class")?,
            )),
            TypeKind::Class(struct_decl) => Ok(TypeKind::Class(
                struct_decl
                    .replace_template_parameters(types, get_param_type)
                    .or_raise_str(|| "Failed to specialize class inside template class")?,
            )),
            TypeKind::Union(union_decl) => Ok(TypeKind::Union(
                union_decl
                    .replace_template_parameters(types, get_param_type)
                    .or_raise_str(|| "Failed to specialize union inside template class")?,
            )),
            TypeKind::Enum(enum_decl) => Ok(TypeKind::Enum(enum_decl.clone())),
            TypeKind::Typedef(typedef) => Ok(TypeKind::Typedef(Box::new(
                typedef
                    .replace_template_parameters(types, get_param_type)
                    .or_raise_str(|| "Failed to specialize typedef inside template class")?,
            ))),
            TypeKind::Named(type_path) => {
                let named_type = types
                    .get(type_path.clone())
                    .ok_or_raise_str(|| format!("Named type not found: {}", type_path))?;
                named_type.replace_template_parameters(types, get_param_type)
            }
            TypeKind::TemplateParam(name) => {
                if let Some(ty) = get_param_type(name) {
                    Ok(ty)
                } else {
                    Ok(TypeKind::TemplateParam(name.clone()))
                }
            }
            TypeKind::TemplateClassSpec(specialized) => Ok(TypeKind::TemplateClassSpec(
                specialized
                    .replace_template_parameters(types, get_param_type)
                    .or_raise_str(|| "Failed to specialize template class inside template class")?,
            )),
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
            TypeKind::TemplateParam(name) => write!(f, "typename {name}"),
            TypeKind::TemplateClassSpec(specialized) => write!(f, "specialized {specialized}"),
        }
    }
}
