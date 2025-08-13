use crate::{
    EnumDecl, Env, StructDecl, TypeDecl, Typedef, Types, UnionDecl,
    error::{InvalidAstSnafu, ParseError, UnsupportedEntitySnafu},
};

pub struct Parser {
    types: Types,
}

impl Parser {
    pub fn new() -> Self {
        Parser { types: Types::new() }
    }

    pub fn into_types(self) -> Types {
        self.types
    }

    fn parse_children(&mut self, env: &Env, node: &clang::Entity) -> Result<(), ParseError> {
        for child in node.get_children() {
            self.parse(env, &child)?;
        }
        Ok(())
    }

    pub(crate) fn parse(&mut self, env: &Env, node: &clang::Entity) -> Result<(), ParseError> {
        let kind = node.get_kind();
        if kind != clang::EntityKind::NotImplemented
            && !node.get_location().unwrap().is_in_main_file()
        {
            // Skip entities not in the main file
            return Ok(());
        }

        match node.get_kind() {
            clang::EntityKind::NotImplemented => self.parse_children(env, node)?,
            // typedef <underlying_type> <name>;
            clang::EntityKind::TypedefDecl => {
                if let Some(child) = node.get_child(0)
                    && child.get_kind() == clang::EntityKind::StructDecl
                {
                    // Skip "typedef struct" declarations
                    return Ok(());
                }
                let underlying_type = node.get_typedef_underlying_type().ok_or_else(|| {
                    InvalidAstSnafu {
                        message: format!("TypedefDecl without underlying type: {node:?}"),
                    }
                    .build()
                })?;
                let name = node.get_name().ok_or_else(|| {
                    InvalidAstSnafu { message: format!("TypedefDecl without name: {node:?}") }
                        .build()
                })?;
                let typedef = Typedef::new(env, &self.types, name, underlying_type)?;
                self.types.add_type(TypeDecl::Typedef(typedef));
            }
            clang::EntityKind::EnumDecl => {
                let name = node.get_name();
                let enum_decl = EnumDecl::new(name, node)?;
                self.types.add_type(TypeDecl::Enum(enum_decl));
            }
            clang::EntityKind::StructDecl | clang::EntityKind::ClassDecl => {
                let name = node.get_name().ok_or_else(|| {
                    InvalidAstSnafu { message: format!("StructDecl without name: {node:?}") }
                        .build()
                })?;
                let ty = node.get_type().ok_or_else(|| {
                    InvalidAstSnafu { message: format!("StructDecl without type: {node:?}") }
                        .build()
                })?;
                let struct_decl = StructDecl::new(env, &self.types, Some(name), ty)?;
                self.types.add_type(TypeDecl::Struct(struct_decl));
            }
            clang::EntityKind::Namespace => {
                self.parse_children(env, node)?;
            }
            clang::EntityKind::LinkageSpec => {
                self.parse_children(env, node)?;
            }
            clang::EntityKind::ClassTemplate => {
                // TODO: Handle template classes
            }
            clang::EntityKind::UnionDecl => {
                let name = node.get_name().ok_or_else(|| {
                    InvalidAstSnafu { message: format!("UnionDecl without name: {node:?}") }.build()
                })?;
                let ty = node.get_type().ok_or_else(|| {
                    InvalidAstSnafu { message: format!("UnionDecl without type: {node:?}") }.build()
                })?;
                let union_decl = UnionDecl::new(env, &self.types, Some(name), ty)?;
                self.types.add_type(TypeDecl::Union(union_decl));
            }

            clang::EntityKind::FunctionDecl => {}
            clang::EntityKind::VarDecl => {}
            clang::EntityKind::UnexposedDecl => {}
            clang::EntityKind::UsingDeclaration => {}
            _ => {
                return UnsupportedEntitySnafu {
                    at: "global scope".to_string(),
                    message: format!("Unsupported entity kind: {:?}", node.get_kind()),
                }
                .fail();
            }
        }
        Ok(())
    }
}
