use crate::{EnumDecl, StructDecl, Typedef, Types, error::ParseError};

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

    fn parse_children(&mut self, node: &clang::Entity) -> Result<(), ParseError> {
        for child in node.get_children() {
            self.parse(&child)?;
        }
        Ok(())
    }

    pub(crate) fn parse(&mut self, node: &clang::Entity) -> Result<(), ParseError> {
        match node.get_kind() {
            clang::EntityKind::NotImplemented => self.parse_children(node)?,
            // typedef <underlying_type> <name>;
            clang::EntityKind::TypedefDecl => {
                if let Some(child) = node.get_child(0)
                    && child.get_kind() == clang::EntityKind::StructDecl
                {
                    // Skip "typedef struct" declarations
                    return Ok(());
                }
                let underlying_type = node.get_typedef_underlying_type().ok_or_else(|| {
                    ParseError::InvalidAst(format!("TypedefDecl without underlying type: {node:?}"))
                })?;
                let name = node.get_name().ok_or_else(|| {
                    ParseError::InvalidAst(format!("TypedefDecl without name: {node:?}"))
                })?;
                let typedef = Typedef::new(name, underlying_type)?;
                self.types.add_typedef(typedef);
            }
            clang::EntityKind::EnumDecl => {
                let name = node.get_name().ok_or_else(|| {
                    ParseError::InvalidAst(format!("EnumDecl without name: {node:?}"))
                })?;
                let enum_decl = EnumDecl::new(name, node)?;
                self.types.add_enum(enum_decl);
            }
            clang::EntityKind::StructDecl | clang::EntityKind::ClassDecl => {
                let name = node.get_name().ok_or_else(|| {
                    ParseError::InvalidAst(format!("StructDecl without name: {node:?}"))
                })?;
                let struct_decl = StructDecl::new(name, node)?;
                self.types.add_struct(struct_decl);
            }
            clang::EntityKind::Namespace => {
                self.parse_children(node)?;
            }
            clang::EntityKind::LinkageSpec => {
                self.parse_children(node)?;
            }
            clang::EntityKind::ClassTemplate => {
                // TODO: Handle template classes
            }

            clang::EntityKind::FunctionDecl => {}
            clang::EntityKind::VarDecl => {}
            clang::EntityKind::UnexposedDecl => {}
            clang::EntityKind::UsingDeclaration => {}
            _ => {
                return Err(ParseError::UnsupportedEntity {
                    at: "global scope".to_string(),
                    message: format!("Unsupported entity kind: {:?}", node.get_kind()),
                });
            }
        }
        Ok(())
    }
}
