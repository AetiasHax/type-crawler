use std::{collections::HashSet, path::PathBuf};

use crate::{
    EnumDecl, Env, StructDecl, TypeKind, TypePath, Typedef, Types, UnionDecl,
    error::{InvalidAstSnafu, ParseError, UnsupportedEntitySnafu},
};

pub struct Parser {
    types: Types,
    crawled_files: HashSet<PathBuf>,
    new_includes: Vec<PathBuf>,
}

impl Parser {
    pub fn new() -> Self {
        Parser { types: Types::new(), crawled_files: HashSet::new(), new_includes: Vec::new() }
    }

    pub fn into_types(self) -> Types {
        self.types
    }

    fn parse_children(&mut self, env: &Env, node: &clang::Entity) -> Result<(), ParseError> {
        for child in node.get_children() {
            self.do_parse(env, &child)?;
        }
        Ok(())
    }

    pub(crate) fn mark_as_crawled(&mut self, file: PathBuf) {
        self.crawled_files.insert(file);
    }

    pub(crate) fn parse(&mut self, env: &Env, node: &clang::Entity) -> Result<(), ParseError> {
        debug_assert!(self.new_includes.is_empty());
        self.do_parse(env, node)?;
        for new_include in self.new_includes.extract_if(.., |_| true) {
            self.crawled_files.insert(new_include);
        }
        Ok(())
    }

    fn do_parse(&mut self, env: &Env, node: &clang::Entity) -> Result<(), ParseError> {
        let kind = node.get_kind();
        if let Some(location) = node.get_location()
            && let Some(file) = &location.get_file_location().file
            && self.crawled_files.contains(&file.get_path())
        {
            // Skip already processed files
            return Ok(());
        }

        match kind {
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
                let path = TypePath::from_entity(node)?;
                let typedef = Typedef::new(env, &self.types, path, underlying_type)?;
                self.types.add_type(TypeKind::Typedef(Box::new(typedef)))?;
            }
            clang::EntityKind::EnumDecl => {
                let path = TypePath::from_entity(node)?;
                let enum_decl = EnumDecl::new(Some(path), node)?;
                self.types.add_type(TypeKind::Enum(enum_decl))?;
            }
            clang::EntityKind::StructDecl => {
                let path = TypePath::from_entity(node)?;
                let struct_decl = StructDecl::new(env, &self.types, Some(path), node)?;
                self.types.add_type(TypeKind::Struct(struct_decl))?;
            }
            clang::EntityKind::ClassDecl => {
                let path = TypePath::from_entity(node)?;
                let class_decl = StructDecl::new(env, &self.types, Some(path), node)?;
                self.types.add_type(TypeKind::Class(class_decl))?;
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
                let path = TypePath::from_entity(node)?;
                let ty = node.get_type().ok_or_else(|| {
                    InvalidAstSnafu { message: format!("UnionDecl without type: {node:?}") }.build()
                })?;
                let union_decl = UnionDecl::new(env, &self.types, Some(path), ty)?;
                self.types.add_type(TypeKind::Union(union_decl))?;
            }

            clang::EntityKind::FunctionDecl => {}
            clang::EntityKind::VarDecl => {}
            clang::EntityKind::UnexposedDecl => {}
            clang::EntityKind::UsingDeclaration => {}
            clang::EntityKind::MacroDefinition => {}
            clang::EntityKind::InclusionDirective => {
                let file = node.get_file().ok_or_else(|| {
                    InvalidAstSnafu { message: "InclusionDirective without a file" }.build()
                })?;
                self.new_includes.push(file.get_path());
            }
            clang::EntityKind::MacroExpansion => {}
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
