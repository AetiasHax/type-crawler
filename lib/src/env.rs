pub struct Env {
    word_size: WordSize,
}

impl Env {
    pub fn new(word_size: WordSize) -> Self {
        Env { word_size }
    }

    pub fn word_size(&self) -> &WordSize {
        &self.word_size
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

    pub fn clang_arg(&self) -> &'static str {
        match self {
            WordSize::Size16 => "-m16",
            WordSize::Size32 => "-m32",
            WordSize::Size64 => "-m64",
        }
    }
}
