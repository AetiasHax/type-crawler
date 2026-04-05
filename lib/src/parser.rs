use std::{collections::HashSet, path::PathBuf};

use crate::{
    EnumDecl, Env, StructDecl, TemplateClass, TypeKind, TypePath, Typedef, Types, UnionDecl,
    error::{ExnExt, OptionExt as _, bail_str, error_type},
};

error_type!(ParserError);

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

    fn parse_children(&mut self, env: &Env, node: &clang::Entity) -> exn::Result<(), ParserError> {
        for child in node.get_children() {
            self.do_parse(env, &child)?;
        }
        Ok(())
    }

    pub(crate) fn mark_as_crawled(&mut self, file: PathBuf) {
        self.crawled_files.insert(file);
    }

    pub(crate) fn parse(
        &mut self,
        env: &Env,
        node: &clang::Entity,
    ) -> exn::Result<(), ParserError> {
        debug_assert!(self.new_includes.is_empty());
        self.do_parse(env, node)?;
        for new_include in self.new_includes.extract_if(.., |_| true) {
            self.crawled_files.insert(new_include);
        }
        Ok(())
    }

    fn do_parse(&mut self, env: &Env, node: &clang::Entity) -> exn::Result<(), ParserError> {
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
                let underlying_type = node.get_typedef_underlying_type().ok_or_raise_str(|| {
                    format!("TypedefDecl without underlying type: {:?}", node)
                })?;
                let path =
                    TypePath::from_entity(node).or_raise_str(|| "Failed to get path to typedef")?;
                let typedef = Typedef::new(env, &self.types, path, underlying_type)
                    .or_raise_str(|| "Failed to parse typedef AST")?;
                self.types
                    .add_type(TypeKind::Typedef(Box::new(typedef)))
                    .or_raise_str(|| "Failed to add typedef to types list")?;
            }
            clang::EntityKind::EnumDecl => {
                let path =
                    TypePath::from_entity(node).or_raise_str(|| "Failed to get path to enum")?;
                let enum_decl =
                    EnumDecl::new(Some(path), node).or_raise_str(|| "Failed to parse enum AST")?;
                self.types
                    .add_type(TypeKind::Enum(enum_decl))
                    .or_raise_str(|| "Failed to add enum to types list")?;
            }
            clang::EntityKind::StructDecl => {
                let path =
                    TypePath::from_entity(node).or_raise_str(|| "Failed to get path to struct")?;
                let struct_decl = StructDecl::new(env, &self.types, Some(path), node)
                    .or_raise_str(|| "Failed to parse struct AST")?;
                self.types
                    .add_type(TypeKind::Struct(struct_decl))
                    .or_raise_str(|| "Failed to add struct to types list")?;
            }
            clang::EntityKind::ClassDecl => {
                let path =
                    TypePath::from_entity(node).or_raise_str(|| "Failed to get path to class")?;
                let class_decl = StructDecl::new(env, &self.types, Some(path), node)
                    .or_raise_str(|| "Failed to parse class AST")?;
                self.types
                    .add_type(TypeKind::Class(class_decl))
                    .or_raise_str(|| "Failed to add class to types list")?;
            }
            clang::EntityKind::Namespace => {
                self.parse_children(env, node)?;
            }
            clang::EntityKind::LinkageSpec => {
                self.parse_children(env, node)?;
            }
            clang::EntityKind::ClassTemplate => {
                let path = TypePath::from_entity(node)
                    .or_raise_str(|| "Failed to get path to template class")?;
                let template_class = TemplateClass::new(env, &self.types, path, node)
                    .or_raise_str(|| "Failed to parse template class AST")?;
                self.types
                    .add_template_class(template_class)
                    .or_raise_str(|| "Failed to add to template class list")?;
            }
            clang::EntityKind::UnionDecl => {
                let path =
                    TypePath::from_entity(node).or_raise_str(|| "Failed to get path to union")?;
                let ty = node
                    .get_type()
                    .ok_or_raise_str(|| format!("UnionDecl without type: {node:?}"))?;
                let union_decl = UnionDecl::new(env, &self.types, Some(path), ty)
                    .or_raise_str(|| "Failed to parse union AST")?;
                self.types
                    .add_type(TypeKind::Union(union_decl))
                    .or_raise_str(|| "Failed to add union to types list")?;
            }

            clang::EntityKind::FunctionDecl => {}
            clang::EntityKind::VarDecl => {}
            clang::EntityKind::UnexposedDecl => {}
            clang::EntityKind::UsingDeclaration => {}
            clang::EntityKind::MacroDefinition => {}
            clang::EntityKind::InclusionDirective => {
                let file =
                    node.get_file().ok_or_raise_str(|| "InclusionDirective without a file")?;
                self.new_includes.push(file.get_path());
            }
            clang::EntityKind::MacroExpansion => {}
            _ => {
                bail_str!("Unsupported entity at global scope: {:?}", node);
            }
        }
        Ok(())
    }
}
