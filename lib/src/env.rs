pub struct Env {
    word_size: WordSize,
    short_enums: bool,
    signed_char: bool,
}

pub struct EnvOptions {
    pub word_size: WordSize,
    pub short_enums: bool,
    pub signed_char: bool,
}

impl Env {
    pub fn new(options: EnvOptions) -> Self {
        let EnvOptions { word_size, short_enums, signed_char } = options;
        Env { word_size, short_enums, signed_char }
    }

    pub fn word_size(&self) -> &WordSize {
        &self.word_size
    }

    pub fn short_enums_clang_arg(&self) -> &'static str {
        if self.short_enums { "-fshort-enums" } else { "-fno-short-enums" }
    }

    pub fn signed_char_clang_arg(&self) -> &'static str {
        if self.signed_char { "-fsigned-char" } else { "-funsigned-char" }
    }
}

impl Default for EnvOptions {
    fn default() -> Self {
        EnvOptions { word_size: WordSize::Size64, short_enums: true, signed_char: true }
    }
}

pub enum WordSize {
    Size16,
    Size32,
    Size64,
}

impl WordSize {
    pub fn bits(&self) -> usize {
        match self {
            WordSize::Size16 => 16,
            WordSize::Size32 => 32,
            WordSize::Size64 => 64,
        }
    }

    pub fn bytes(&self) -> usize {
        match self {
            WordSize::Size16 => 2,
            WordSize::Size32 => 4,
            WordSize::Size64 => 8,
        }
    }

    pub fn clang_arg(&self) -> &'static str {
        match self {
            WordSize::Size16 => "-m16",
            WordSize::Size32 => "-m32",
            WordSize::Size64 => "-m64",
        }
    }
}
