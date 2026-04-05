use std::path::{Path, PathBuf};

use clang::Clang;

use crate::{
    Env, Types,
    error::{
        AddIncludePathError, ClangInitSnafu, DoesNotExistSnafu, FileNotFoundSnafu,
        NotADirectorySnafu, ParseError, TypeCrawlerError,
    },
    parser::Parser,
};

pub struct TypeCrawler {
    clang: Clang,
    include_paths: Vec<PathBuf>,
    env: Env,
    ast_parser: Parser,
}

impl TypeCrawler {
    pub fn new(env: Env) -> Result<Self, TypeCrawlerError> {
        let clang = Clang::new().map_err(|message| ClangInitSnafu { message }.build())?;
        Ok(Self::from_clang(clang, env))
    }

    pub fn from_clang(clang: Clang, env: Env) -> Self {
        TypeCrawler { clang, include_paths: Vec::new(), env, ast_parser: Parser::new() }
    }

    pub fn into_types(self) -> Types {
        self.ast_parser.into_types()
    }

    pub fn add_include_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), AddIncludePathError> {
        let path = path.as_ref();
        if !path.exists() {
            return DoesNotExistSnafu { path: path.display().to_string() }.fail();
        }
        if !path.is_dir() {
            return NotADirectorySnafu { path: path.display().to_string() }.fail();
        }

        let path_buf = path.to_path_buf();
        if !self.include_paths.contains(&path_buf) {
            self.include_paths.push(path_buf);
        }
        Ok(())
    }

    fn arguments(&self) -> Vec<String> {
        self.include_paths
            .iter()
            .map(|p| format!("-I{}", p.display()))
            .chain([
                self.env.word_size().clang_arg().to_string(),
                self.env.short_enums_clang_arg().to_string(),
                self.env.signed_char_clang_arg().to_string(),
                "-nostdinc".to_string(),
            ])
            .collect()
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), ParseError> {
        let path = file_path.as_ref();
        if !path.exists() {
            return FileNotFoundSnafu { name: path.display().to_string() }.fail();
        }

        let index = clang::Index::new(&self.clang, false, false);
        let mut clang_parser = index.parser(path);
        clang_parser.skip_function_bodies(true); // only function declarations needed
        clang_parser.detailed_preprocessing_record(true); // process macros, notably #include
        let mut arguments = self.arguments();
        if path.extension().is_none() {
            // Assume C++ for headers like `vector`, `string`, etc.
            arguments.push("-x".into());
            arguments.push("c++".into());
        }
        clang_parser.arguments(&arguments);
        let unit = clang_parser.parse()?;

        let root = unit.get_entity();

        self.ast_parser.parse(&self.env, &root)?;
        self.ast_parser.mark_as_crawled(path.to_path_buf());

        Ok(())
    }

    pub fn print_file_ast<P: AsRef<Path>>(&self, file_path: P) -> Result<(), ParseError> {
        let path = file_path.as_ref();
        let index = clang::Index::new(&self.clang, false, false);
        let mut parser = index.parser(path);
        parser.skip_function_bodies(true);
        parser.detailed_preprocessing_record(true);
        parser.arguments(&self.arguments());
        let unit = parser.parse()?;

        let root = unit.get_entity();
        Self::display_ast(&root, 0, false);
        Ok(())
    }

    fn display_ast(entity: &clang::Entity, indent: usize, argument: bool) {
        let indent_str = " ".repeat(indent);
        print!("{}{:?} {}", indent_str, entity.get_kind(), entity.get_name().unwrap_or_default());

        if entity.is_virtual_method() {
            print!(" virtual");
        }

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

        if let Some(file) = entity.get_file() {
            print!(" at {}", file.get_path().display());
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
