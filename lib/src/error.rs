use clang::SourceError;
use snafu::Snafu;

use crate::ExtendTypesError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum TypeCrawlerError {
    #[snafu(display("Failed to initialize clang: {message}"))]
    ClangInit { message: String },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AddIncludePathError {
    #[snafu(display("Path does not exist: {path}"))]
    DoesNotExist { path: String },
    #[snafu(display("Path is not a directory: {path}"))]
    NotADirectory { path: String },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ParseError {
    #[snafu(display("File not found: {name}"))]
    FileNotFound { name: String },
    #[snafu(display("Failed to read file: {path}"))]
    ReadError { path: String },
    #[snafu(transparent)]
    ParseError { source: SourceError },
    #[snafu(display("Invalid AST: {message}"))]
    InvalidAst { message: String },
    #[snafu(display("Unsupported type: {message}"))]
    UnsupportedType { message: String },
    #[snafu(display("Unsupported entity in {at}: {message}"))]
    UnsupportedEntity { at: String, message: String },
    #[snafu(display("Failed to get field offset of {field_name} in {struct_name}: {error}"))]
    Offsetof { field_name: String, struct_name: String, error: clang::OffsetofError },
    #[snafu(display("Failed to get size of type {type_name}: {error}"))]
    Sizeof { type_name: String, error: clang::SizeofError },
    #[snafu(display("Failed to get alignment of type {type_name}: {error}"))]
    Alignof { type_name: String, error: clang::AlignofError },
    #[snafu(display("Invalid fields in {struct_name}: {field_names:?}"))]
    InvalidFields { field_names: Vec<String>, struct_name: String },
    #[snafu(transparent)]
    ExtendTypesError { source: ExtendTypesError },
}
