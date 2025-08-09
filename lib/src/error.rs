use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeCrawlerError {
    #[error("Failed to initialize clang: {0}")]
    ClangInit(String),
}

#[derive(Error, Debug)]
pub enum AddIncludePathError {
    #[error("Path does not exist: {0}")]
    DoesNotExist(String),
    #[error("Path is not a directory: {0}")]
    NotADirectory(String),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Failed to read file: {0}")]
    ReadError(String),
    #[error("Failed to parse file: {0}")]
    ParseError(#[from] clang::SourceError),
    #[error("Invalid AST: {0}")]
    InvalidAst(String),
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
    #[error("Unsupported entity in {at}: {message}")]
    UnsupportedEntity { at: String, message: String },
    #[error("Failed to get field offset of {field_name} in {struct_name}: {error}")]
    Offsetof { field_name: String, struct_name: String, error: clang::OffsetofError },
    #[error("Failed to get size of type: {0}")]
    Sizeof(#[from] clang::SizeofError),
}
