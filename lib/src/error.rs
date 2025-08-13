use std::backtrace::Backtrace;

use clang::SourceError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum TypeCrawlerError {
    #[snafu(display("Failed to initialize clang: {message}:\n{backtrace}"))]
    ClangInit { message: String, backtrace: Backtrace },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AddIncludePathError {
    #[snafu(display("Path does not exist: {path}:\n{backtrace}"))]
    DoesNotExist { path: String, backtrace: Backtrace },
    #[snafu(display("Path is not a directory: {path}:\n{backtrace}"))]
    NotADirectory { path: String, backtrace: Backtrace },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ParseError {
    #[snafu(display("File not found: {name}:\n{backtrace}"))]
    FileNotFound { name: String, backtrace: Backtrace },
    #[snafu(display("Failed to read file: {path}:\n{backtrace}"))]
    ReadError { path: String, backtrace: Backtrace },
    #[snafu(transparent)]
    ParseError { source: SourceError, backtrace: Backtrace },
    #[snafu(display("Invalid AST: {message}:\n{backtrace}"))]
    InvalidAst { message: String, backtrace: Backtrace },
    #[snafu(display("Unsupported type: {message}:\n{backtrace}"))]
    UnsupportedType { message: String, backtrace: Backtrace },
    #[snafu(display("Unsupported entity in {at}: {message}:\n{backtrace}"))]
    UnsupportedEntity { at: String, message: String, backtrace: Backtrace },
    #[snafu(display(
        "Failed to get field offset of {field_name} in {struct_name}: {error}:\n{backtrace}"
    ))]
    Offsetof {
        field_name: String,
        struct_name: String,
        error: clang::OffsetofError,
        backtrace: Backtrace,
    },
    #[snafu(display("Failed to get size of type {type_name}: {error}:\n{backtrace}"))]
    Sizeof { type_name: String, error: clang::SizeofError, backtrace: Backtrace },
    #[snafu(display("Failed to get alignment of type {type_name}: {error}:\n{backtrace}"))]
    Alignof { type_name: String, error: clang::AlignofError, backtrace: Backtrace },
    #[snafu(display("Invalid fields in {struct_name}: {field_names:?}:\n{backtrace}"))]
    InvalidFields { field_names: Vec<String>, struct_name: String, backtrace: Backtrace },
}
