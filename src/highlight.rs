use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "AND", "OR", "NOT", "IN", "IS", "NULL", "LIKE",
    "BETWEEN", "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END", "AS", "ON",
    "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "CROSS", "NATURAL",
    "ORDER", "BY", "ASC", "DESC", "GROUP", "HAVING", "LIMIT", "OFFSET",
    "UNION", "ALL", "INTERSECT", "EXCEPT", "DISTINCT", "INTO", "VALUES",
    "INSERT", "UPDATE", "DELETE", "SET", "CREATE", "TABLE", "INDEX", "VIEW",
    "DROP", "ALTER", "ADD", "COLUMN", "PRIMARY", "KEY", "FOREIGN", "REFERENCES",
    "UNIQUE", "CHECK", "DEFAULT", "CONSTRAINT", "CASCADE", "RESTRICT",
    "PRAGMA", "EXPLAIN", "ANALYZE", "VACUUM", "ATTACH", "DETACH", "BEGIN",
    "COMMIT", "ROLLBACK", "TRANSACTION", "SAVEPOINT", "RELEASE", "IF",
    "REPLACE", "ABORT", "FAIL", "IGNORE", "CONFLICT", "COLLATE", "GLOB",
    "REGEXP", "MATCH", "ESCAPE", "INDEXED", "REINDEX", "RENAME", "TO",
    "TEMP", "TEMPORARY", "TRIGGER", "AFTER", "BEFORE", "INSTEAD", "OF",
    "FOR", "EACH", "ROW", "RECURSIVE", "WITH", "WITHOUT", "ROWID",
    "TRUE", "FALSE", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP",
];

const SQL_FUNCTIONS: &[&str] = &[
    "COUNT", "SUM", "AVG", "MIN", "MAX", "ABS", "COALESCE", "NULLIF",
    "IFNULL", "IIF", "INSTR", "LENGTH", "LOWER", "UPPER", "LTRIM", "RTRIM",
    "TRIM", "REPLACE", "SUBSTR", "SUBSTRING", "PRINTF", "TYPEOF", "UNICODE",
    "ZEROBLOB", "DATE", "TIME", "DATETIME", "JULIANDAY", "STRFTIME",
    "RANDOM", "RANDOMBLOB", "HEX", "UNHEX", "QUOTE", "TOTAL", "GROUP_CONCAT",
    "CAST", "ROUND", "LIKELY", "UNLIKELY", "LOAD_EXTENSION", "JSON",
    "JSON_ARRAY", "JSON_OBJECT", "JSON_EXTRACT", "JSON_TYPE", "JSON_VALID",
];

const SQL_TYPES: &[&str] = &[
    "INTEGER", "INT", "TINYINT", "SMALLINT", "MEDIUMINT", "BIGINT",
    "UNSIGNED", "INT2", "INT8", "TEXT", "CLOB", "BLOB", "REAL", "DOUBLE",
    "FLOAT", "NUMERIC", "DECIMAL", "BOOLEAN", "DATE", "DATETIME",
    "VARCHAR", "CHAR", "NCHAR", "NVARCHAR",
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    Keyword,
    Function,
    Type,
    String,
    Number,
    Comment,
    Operator,
    Identifier,
    Whitespace,
}

#[derive(Debug)]
struct Token {
    text: String,
    token_type: TokenType,
}

pub struct SqlHighlighter {
    keyword_style: Style,
    function_style: Style,
    type_style: Style,
    string_style: Style,
    number_style: Style,
    comment_style: Style,
    operator_style: Style,
    default_style: Style,
}

impl Default for SqlHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlHighlighter {
    pub fn new() -> Self {
        Self {
            keyword_style: Style::default().fg(Color::Magenta),
            function_style: Style::default().fg(Color::Cyan),
            type_style: Style::default().fg(Color::Yellow),
            string_style: Style::default().fg(Color::Green),
            number_style: Style::default().fg(Color::LightBlue),
            comment_style: Style::default().fg(Color::DarkGray),
            operator_style: Style::default().fg(Color::White),
            default_style: Style::default().fg(Color::White),
        }
    }

    pub fn highlight_line(&self, line: &str) -> Line<'static> {
        let tokens = self.tokenize(line);
        let spans: Vec<Span> = tokens
            .into_iter()
            .map(|token| {
                let style = match token.token_type {
                    TokenType::Keyword => self.keyword_style,
                    TokenType::Function => self.function_style,
                    TokenType::Type => self.type_style,
                    TokenType::String => self.string_style,
                    TokenType::Number => self.number_style,
                    TokenType::Comment => self.comment_style,
                    TokenType::Operator => self.operator_style,
                    TokenType::Identifier | TokenType::Whitespace => self.default_style,
                };
                Span::styled(token.text, style)
            })
            .collect();
        Line::from(spans)
    }

    fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            // Whitespace
            if ch.is_whitespace() {
                let start = i;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                tokens.push(Token {
                    text: chars[start..i].iter().collect(),
                    token_type: TokenType::Whitespace,
                });
                continue;
            }

            // Single-line comment (--)
            if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
                let text: String = chars[i..].iter().collect();
                tokens.push(Token {
                    text,
                    token_type: TokenType::Comment,
                });
                break;
            }

            // Block comment start (/* ... */)
            if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                let start = i;
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                if i + 1 < chars.len() {
                    i += 2; // skip */
                }
                tokens.push(Token {
                    text: chars[start..i].iter().collect(),
                    token_type: TokenType::Comment,
                });
                continue;
            }

            // String literals
            if ch == '\'' || ch == '"' {
                let quote = ch;
                let start = i;
                i += 1;
                while i < chars.len() {
                    if chars[i] == quote {
                        if i + 1 < chars.len() && chars[i + 1] == quote {
                            i += 2; // escaped quote
                        } else {
                            i += 1;
                            break;
                        }
                    } else {
                        i += 1;
                    }
                }
                tokens.push(Token {
                    text: chars[start..i].iter().collect(),
                    token_type: TokenType::String,
                });
                continue;
            }

            // Numbers
            if ch.is_ascii_digit() || (ch == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                // Handle scientific notation
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    i += 1;
                    if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                        i += 1;
                    }
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                tokens.push(Token {
                    text: chars[start..i].iter().collect(),
                    token_type: TokenType::Number,
                });
                continue;
            }

            // Identifiers and keywords
            if ch.is_alphabetic() || ch == '_' || ch == '`' || ch == '[' {
                let start = i;
                let is_quoted = ch == '`' || ch == '[';
                let end_quote = if ch == '[' { ']' } else { '`' };

                if is_quoted {
                    i += 1;
                    while i < chars.len() && chars[i] != end_quote {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1;
                    }
                    tokens.push(Token {
                        text: chars[start..i].iter().collect(),
                        token_type: TokenType::Identifier,
                    });
                } else {
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let text: String = chars[start..i].iter().collect();
                    let upper = text.to_uppercase();
                    let token_type = if SQL_KEYWORDS.contains(&upper.as_str()) {
                        TokenType::Keyword
                    } else if SQL_FUNCTIONS.contains(&upper.as_str()) {
                        TokenType::Function
                    } else if SQL_TYPES.contains(&upper.as_str()) {
                        TokenType::Type
                    } else {
                        TokenType::Identifier
                    };
                    tokens.push(Token { text, token_type });
                }
                continue;
            }

            // Operators and punctuation
            let op_text = match ch {
                '(' | ')' | ',' | ';' | '.' | '*' | '+' | '-' | '/' | '%' | '=' | '<' | '>'
                | '!' | '&' | '|' | '^' | '~' => {
                    // Check for multi-char operators
                    if i + 1 < chars.len() {
                        let next = chars[i + 1];
                        match (ch, next) {
                            ('<', '=') | ('>', '=') | ('!', '=') | ('<', '>') | ('|', '|')
                            | ('<', '<') | ('>', '>') => {
                                i += 2;
                                format!("{}{}", ch, next)
                            }
                            _ => {
                                i += 1;
                                ch.to_string()
                            }
                        }
                    } else {
                        i += 1;
                        ch.to_string()
                    }
                }
                _ => {
                    i += 1;
                    ch.to_string()
                }
            };

            tokens.push(Token {
                text: op_text,
                token_type: TokenType::Operator,
            });
        }

        tokens
    }
}
