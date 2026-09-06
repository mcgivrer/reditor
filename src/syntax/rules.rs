/// Configuration d'un langage "générique" (style C) pour le tokenizer
/// partagé dans `generic.rs`.
pub struct GenericLangConfig {
    pub line_comment: &'static [&'static str],
    pub block_comment: Option<(&'static str, &'static str)>,
    pub string_quotes: &'static [char],
    pub keywords: &'static [&'static str],
    pub types_or_builtins: &'static [&'static str],
    pub booleans: &'static [&'static str],
    pub variable_sigil: Option<char>,
}

const EMPTY: &[&str] = &[];

pub fn rust_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["//"],
        block_comment: Some(("/*", "*/")),
        string_quotes: &['"', '\''],
        keywords: &[
            "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
            "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
            "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
            "true", "type", "unsafe", "use", "where", "while", "async", "await", "move", "yield",
        ],
        types_or_builtins: &[
            "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
            "usize", "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result",
            "Box", "Rc", "Arc", "HashMap", "HashSet", "BTreeMap",
        ],
        booleans: &["true", "false"],
        variable_sigil: None,
    }
}

pub fn java_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["//"],
        block_comment: Some(("/*", "*/")),
        string_quotes: &['"', '\''],
        keywords: &[
            "abstract", "assert", "break", "case", "catch", "class", "const", "continue",
            "default", "do", "else", "enum", "extends", "final", "finally", "for", "goto", "if",
            "implements", "import", "instanceof", "interface", "native", "new", "package",
            "private", "protected", "public", "return", "static", "strictfp", "super", "switch",
            "synchronized", "this", "throw", "throws", "transient", "try", "void", "volatile",
            "while", "var", "record", "sealed", "permits", "yield",
        ],
        types_or_builtins: &[
            "boolean", "byte", "char", "double", "float", "int", "long", "short", "String",
            "Object", "Integer", "Long", "Double", "Float", "Boolean", "List", "Map", "Set",
            "ArrayList", "HashMap", "HashSet",
        ],
        booleans: &["true", "false", "null"],
        variable_sigil: None,
    }
}

pub fn kotlin_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["//"],
        block_comment: Some(("/*", "*/")),
        string_quotes: &['"', '\''],
        keywords: &[
            "as", "break", "class", "companion", "continue", "do", "else", "false", "for",
            "fun", "if", "in", "interface", "internal", "is", "lateinit", "null", "object",
            "override", "package", "private", "protected", "public", "return", "sealed", "super",
            "this", "throw", "true", "try", "typealias", "val", "var", "when", "while", "by",
            "constructor", "data", "enum", "import", "init", "inline", "inner", "open", "operator",
            "out", "reified", "suspend", "vararg", "where", "yield",
        ],
        types_or_builtins: &[
            "Int", "Long", "Short", "Byte", "Float", "Double", "Boolean", "Char", "String",
            "Unit", "Any", "Nothing", "List", "MutableList", "Map", "MutableMap", "Set",
            "MutableSet", "Array",
        ],
        booleans: &["true", "false", "null"],
        variable_sigil: None,
    }
}

pub fn javascript_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["//"],
        block_comment: Some(("/*", "*/")),
        string_quotes: &['"', '\'', '`'],
        keywords: &[
            "async", "await", "break", "case", "catch", "class", "const", "continue", "debugger",
            "default", "delete", "do", "else", "export", "extends", "finally", "for", "function",
            "if", "import", "in", "instanceof", "let", "new", "of", "return", "static", "super",
            "switch", "this", "throw", "try", "typeof", "var", "void", "while", "with", "yield",
            "interface", "type", "enum", "implements", "namespace", "declare", "as", "from",
            "readonly", "public", "private", "protected", "abstract",
        ],
        types_or_builtins: &[
            "Array", "Object", "String", "Number", "Boolean", "Symbol", "Promise", "Map", "Set",
            "Date", "RegExp", "Error", "any", "unknown", "never", "void", "string", "number",
            "boolean",
        ],
        booleans: &["true", "false", "null", "undefined"],
        variable_sigil: None,
    }
}

pub fn c_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["//"],
        block_comment: Some(("/*", "*/")),
        string_quotes: &['"', '\''],
        keywords: &[
            "auto", "break", "case", "const", "continue", "default", "do", "else", "enum",
            "extern", "for", "goto", "if", "inline", "register", "return", "sizeof", "static",
            "struct", "switch", "typedef", "union", "volatile", "while", "include", "define",
            "ifdef", "ifndef", "endif", "pragma",
        ],
        types_or_builtins: &[
            "void", "char", "short", "int", "long", "float", "double", "signed", "unsigned",
            "size_t", "bool",
        ],
        booleans: &["true", "false", "NULL"],
        variable_sigil: None,
    }
}

pub fn cpp_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["//"],
        block_comment: Some(("/*", "*/")),
        string_quotes: &['"', '\''],
        keywords: &[
            "alignas", "alignof", "and", "auto", "break", "case", "catch", "class", "const",
            "constexpr", "continue", "decltype", "default", "delete", "do", "else", "enum",
            "explicit", "export", "extern", "final", "for", "friend", "goto", "if", "inline",
            "mutable", "namespace", "new", "noexcept", "operator", "override", "private",
            "protected", "public", "return", "sizeof", "static", "struct", "switch", "template",
            "this", "throw", "try", "typedef", "typename", "union", "using", "virtual", "void",
            "volatile", "while",
        ],
        types_or_builtins: &[
            "bool", "char", "char16_t", "char32_t", "double", "float", "int", "long", "short",
            "signed", "unsigned", "wchar_t", "std", "string", "vector", "map", "set", "unique_ptr",
            "shared_ptr",
        ],
        booleans: &["true", "false", "nullptr"],
        variable_sigil: None,
    }
}

pub fn go_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["//"],
        block_comment: Some(("/*", "*/")),
        string_quotes: &['"', '\'', '`'],
        keywords: &[
            "break", "case", "chan", "const", "continue", "default", "defer", "else",
            "fallthrough", "for", "func", "go", "goto", "if", "import", "interface", "map",
            "package", "range", "return", "select", "struct", "switch", "type", "var",
        ],
        types_or_builtins: &[
            "bool", "byte", "complex64", "complex128", "error", "float32", "float64", "int",
            "int8", "int16", "int32", "int64", "rune", "string", "uint", "uint8", "uint16",
            "uint32", "uint64", "uintptr",
        ],
        booleans: &["true", "false", "nil", "iota"],
        variable_sigil: None,
    }
}

pub fn python_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["#"],
        block_comment: None,
        string_quotes: &['"', '\''],
        keywords: &[
            "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
            "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in",
            "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
            "with", "yield",
        ],
        types_or_builtins: &[
            "int", "float", "str", "bool", "list", "dict", "set", "tuple", "bytes", "object",
            "type",
        ],
        booleans: &["True", "False", "None"],
        variable_sigil: None,
    }
}

pub fn bash_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: &["#"],
        block_comment: None,
        string_quotes: &['"', '\''],
        keywords: &[
            "if", "then", "elif", "else", "fi", "for", "while", "until", "do", "done", "case",
            "esac", "function", "in", "select", "time", "return", "exit", "break", "continue",
            "local", "readonly", "export", "declare", "unset", "shift", "trap",
        ],
        types_or_builtins: EMPTY,
        booleans: EMPTY,
        variable_sigil: Some('$'),
    }
}

pub fn css_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: EMPTY,
        block_comment: Some(("/*", "*/")),
        string_quotes: &['"', '\''],
        keywords: &[
            "important", "media", "supports", "keyframes", "import", "charset", "font-face",
            "root", "from", "to",
        ],
        types_or_builtins: EMPTY,
        booleans: EMPTY,
        variable_sigil: None,
    }
}

pub fn json_config() -> GenericLangConfig {
    GenericLangConfig {
        line_comment: EMPTY,
        block_comment: None,
        string_quotes: &['"'],
        keywords: EMPTY,
        types_or_builtins: EMPTY,
        booleans: &["true", "false", "null"],
        variable_sigil: None,
    }
}
