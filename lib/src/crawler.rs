use std::path::{Path, PathBuf};

use clang::Clang;

use crate::{
    error::{AddIncludePathError, ParseError, TypeCrawlerError},
    parser::Parser,
    types::Types,
};

pub struct TypeCrawler {
    clang: Clang,
    include_paths: Vec<PathBuf>,
    word_size: WordSize,
}

impl TypeCrawler {
    pub fn new() -> Result<Self, TypeCrawlerError> {
        let clang = Clang::new().map_err(TypeCrawlerError::ClangInit)?;
        Ok(Self::from_clang(clang))
    }

    pub fn from_clang(clang: Clang) -> Self {
        TypeCrawler { clang, include_paths: Vec::new(), word_size: WordSize::Size64 }
    }

    pub fn add_include_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), AddIncludePathError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(AddIncludePathError::DoesNotExist(path.display().to_string()));
        }
        if !path.is_dir() {
            return Err(AddIncludePathError::NotADirectory(path.display().to_string()));
        }

        let path_buf = path.to_path_buf();
        if !self.include_paths.contains(&path_buf) {
            self.include_paths.push(path_buf);
        }
        Ok(())
    }

    pub fn set_word_size(&mut self, word_size: WordSize) {
        self.word_size = word_size;
    }

    pub fn parse_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Types, ParseError> {
        let path = file_path.as_ref();
        if !path.exists() {
            return Err(ParseError::FileNotFound(path.display().to_string()));
        }

        let arguments = self
            .include_paths
            .iter()
            .map(|p| format!("-I{}", p.display()))
            .chain([self.word_size.clang_arg().to_string()])
            .collect::<Vec<_>>();
        println!("{:?}", arguments);

        let index = clang::Index::new(&self.clang, false, false);
        let mut parser = index.parser(path);
        parser.arguments(&arguments);
        let unit = parser.parse()?;

        let root = unit.get_entity();
        Self::display_ast(&root, 0, false);

        let mut context = Parser::new();
        context.parse(&root)?;

        Ok(context.into_types())
    }

    fn display_ast(entity: &clang::Entity, indent: usize, argument: bool) {
        let indent_str = " ".repeat(indent);
        print!("{}{:?} {}", indent_str, entity.get_kind(), entity.get_name().unwrap_or_default());

        let arguments = entity.get_arguments().unwrap_or_default();
        if !arguments.is_empty() {
            println!("(");
            let mut iter = arguments.iter();
            if let Some(first) = iter.next() {
                Self::display_ast(first, indent + 2, true);
            }
            for arg in iter {
                println!(",");
                Self::display_ast(arg, indent + 2, true);
            }
            print!("\n{indent_str})");
        }

        if let Some(underlying_type) = entity.get_typedef_underlying_type() {
            print!(" = {}", underlying_type.get_display_name());
        }

        let children = entity.get_children();
        if !children.is_empty() {
            println!(" {{");
            for child in children {
                Self::display_ast(&child, indent + 2, false);
            }
            print!("{indent_str}}}");
        }
        if !argument {
            println!();
        }
    }
}

pub enum WordSize {
    Size16,
    Size32,
    Size64,
}

impl WordSize {
    pub fn clang_arg(&self) -> &'static str {
        match self {
            WordSize::Size16 => "-m16",
            WordSize::Size32 => "-m32",
            WordSize::Size64 => "-m64",
        }
    }
}
