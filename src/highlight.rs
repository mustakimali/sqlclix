use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

const SQL_KEYWORDS: &[&str] = &[
    // Standard SQL / shared
    "SELECT",
    "FROM",
    "WHERE",
    "AND",
    "OR",
    "NOT",
    "IN",
    "IS",
    "NULL",
    "LIKE",
    "BETWEEN",
    "EXISTS",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "AS",
    "ON",
    "JOIN",
    "LEFT",
    "RIGHT",
    "INNER",
    "OUTER",
    "FULL",
    "CROSS",
    "NATURAL",
    "ORDER",
    "BY",
    "ASC",
    "DESC",
    "GROUP",
    "HAVING",
    "LIMIT",
    "OFFSET",
    "UNION",
    "ALL",
    "INTERSECT",
    "EXCEPT",
    "DISTINCT",
    "INTO",
    "VALUES",
    "INSERT",
    "UPDATE",
    "DELETE",
    "SET",
    "CREATE",
    "TABLE",
    "INDEX",
    "VIEW",
    "DROP",
    "ALTER",
    "ADD",
    "COLUMN",
    "PRIMARY",
    "KEY",
    "FOREIGN",
    "REFERENCES",
    "UNIQUE",
    "CHECK",
    "DEFAULT",
    "CONSTRAINT",
    "CASCADE",
    "RESTRICT",
    "PRAGMA",
    "EXPLAIN",
    "ANALYZE",
    "VACUUM",
    "ATTACH",
    "DETACH",
    "BEGIN",
    "COMMIT",
    "ROLLBACK",
    "TRANSACTION",
    "SAVEPOINT",
    "RELEASE",
    "IF",
    "REPLACE",
    "ABORT",
    "FAIL",
    "IGNORE",
    "CONFLICT",
    "COLLATE",
    "GLOB",
    "REGEXP",
    "MATCH",
    "ESCAPE",
    "INDEXED",
    "REINDEX",
    "RENAME",
    "TO",
    "TEMP",
    "TEMPORARY",
    "TRIGGER",
    "AFTER",
    "BEFORE",
    "INSTEAD",
    "OF",
    "FOR",
    "EACH",
    "ROW",
    "RECURSIVE",
    "WITH",
    "WITHOUT",
    "ROWID",
    "TRUE",
    "FALSE",
    "CURRENT_DATE",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    // PostgreSQL
    "RETURNING",
    "ILIKE",
    "SIMILAR",
    "LATERAL",
    "FETCH",
    "FIRST",
    "NEXT",
    "ROWS",
    "ONLY",
    "WINDOW",
    "PARTITION",
    "OVER",
    "RANGE",
    "UNBOUNDED",
    "PRECEDING",
    "FOLLOWING",
    "CURRENT",
    "EXCLUDE",
    "TIES",
    "OTHERS",
    "NO",
    "ACTION",
    "DEFERRABLE",
    "INITIALLY",
    "DEFERRED",
    "IMMEDIATE",
    "SCHEMA",
    "DATABASE",
    "EXTENSION",
    "SEQUENCE",
    "TYPE",
    "ENUM",
    "DOMAIN",
    "FUNCTION",
    "PROCEDURE",
    "RETURNS",
    "LANGUAGE",
    "PLPGSQL",
    "SQL",
    "IMMUTABLE",
    "STABLE",
    "VOLATILE",
    "SECURITY",
    "DEFINER",
    "INVOKER",
    "PARALLEL",
    "SAFE",
    "UNSAFE",
    "STRICT",
    "CALLED",
    "INPUT",
    "COST",
    "GRANT",
    "REVOKE",
    "PRIVILEGES",
    "OWNER",
    "ROLE",
    "USER",
    "PUBLIC",
    "USAGE",
    "EXECUTE",
    "TRUNCATE",
    "LOCK",
    "SHARE",
    "EXCLUSIVE",
    "ACCESS",
    "NOWAIT",
    "SKIP",
    "LOCKED",
    "MATERIALIZED",
    "REFRESH",
    "CONCURRENTLY",
    "TABLESPACE",
    "UNLOGGED",
    "LOGGED",
    "INHERIT",
    "INHERITS",
    "NOINHERIT",
    "LOGIN",
    "NOLOGIN",
    "SUPERUSER",
    "NOSUPERUSER",
    "CREATEROLE",
    "NOCREATEROLE",
    "CREATEDB",
    "NOCREATEDB",
    "REPLICATION",
    "CONNECTION",
    "NOTIFY",
    "LISTEN",
    "UNLISTEN",
    "COPY",
    "STDIN",
    "STDOUT",
    "DELIMITER",
    "CSV",
    "HEADER",
    "QUOTE",
    "FORCE",
    "FREEZE",
    "VERBOSE",
    "DO",
    "NOTHING",
    "GENERATED",
    "ALWAYS",
    "IDENTITY",
    "OVERRIDING",
    "SYSTEM",
    "VALUE",
    "STORED",
    "INCLUDING",
    "EXCLUDING",
    "LIKE",
    "USING",
    "CLUSTER",
    "COMMENT",
    "DISABLE",
    "ENABLE",
    "RULE",
    "ALSO",
    "FILTER",
    "WITHIN",
    "ORDINALITY",
    "TABLESAMPLE",
    "BERNOULLI",
    "GROUPING",
    "SETS",
    "CUBE",
    "ROLLUP",
];

const SQL_FUNCTIONS: &[&str] = &[
    // Standard / SQLite
    "COUNT",
    "SUM",
    "AVG",
    "MIN",
    "MAX",
    "ABS",
    "COALESCE",
    "NULLIF",
    "IFNULL",
    "IIF",
    "INSTR",
    "LENGTH",
    "LOWER",
    "UPPER",
    "LTRIM",
    "RTRIM",
    "TRIM",
    "REPLACE",
    "SUBSTR",
    "SUBSTRING",
    "PRINTF",
    "TYPEOF",
    "UNICODE",
    "ZEROBLOB",
    "DATE",
    "TIME",
    "DATETIME",
    "JULIANDAY",
    "STRFTIME",
    "RANDOM",
    "RANDOMBLOB",
    "HEX",
    "UNHEX",
    "QUOTE",
    "TOTAL",
    "GROUP_CONCAT",
    "CAST",
    "ROUND",
    "LIKELY",
    "UNLIKELY",
    "LOAD_EXTENSION",
    "JSON",
    "JSON_ARRAY",
    "JSON_OBJECT",
    "JSON_EXTRACT",
    "JSON_TYPE",
    "JSON_VALID",
    // PostgreSQL functions
    "NOW",
    "CURRENT_TIMESTAMP",
    "CLOCK_TIMESTAMP",
    "STATEMENT_TIMESTAMP",
    "TRANSACTION_TIMESTAMP",
    "TIMEOFDAY",
    "AGE",
    "DATE_PART",
    "DATE_TRUNC",
    "EXTRACT",
    "MAKE_DATE",
    "MAKE_TIME",
    "MAKE_TIMESTAMP",
    "MAKE_TIMESTAMPTZ",
    "MAKE_INTERVAL",
    "TO_TIMESTAMP",
    "TO_DATE",
    "TO_CHAR",
    "TO_NUMBER",
    "CONCAT",
    "CONCAT_WS",
    "FORMAT",
    "LEFT",
    "RIGHT",
    "REPEAT",
    "REVERSE",
    "SPLIT_PART",
    "TRANSLATE",
    "INITCAP",
    "LPAD",
    "RPAD",
    "MD5",
    "ENCODE",
    "DECODE",
    "OVERLAY",
    "POSITION",
    "BTRIM",
    "CHR",
    "ASCII",
    "REGEXP_MATCH",
    "REGEXP_MATCHES",
    "REGEXP_REPLACE",
    "REGEXP_SPLIT_TO_ARRAY",
    "REGEXP_SPLIT_TO_TABLE",
    "STRING_AGG",
    "ARRAY_AGG",
    "ARRAY_LENGTH",
    "ARRAY_LOWER",
    "ARRAY_UPPER",
    "ARRAY_DIMS",
    "ARRAY_POSITION",
    "ARRAY_POSITIONS",
    "ARRAY_REMOVE",
    "ARRAY_REPLACE",
    "ARRAY_APPEND",
    "ARRAY_PREPEND",
    "ARRAY_CAT",
    "ARRAY_TO_STRING",
    "STRING_TO_ARRAY",
    "UNNEST",
    "CARDINALITY",
    "GENERATE_SERIES",
    "GENERATE_SUBSCRIPTS",
    "ROW_NUMBER",
    "RANK",
    "DENSE_RANK",
    "PERCENT_RANK",
    "CUME_DIST",
    "NTILE",
    "LAG",
    "LEAD",
    "FIRST_VALUE",
    "LAST_VALUE",
    "NTH_VALUE",
    "BOOL_AND",
    "BOOL_OR",
    "EVERY",
    "BIT_AND",
    "BIT_OR",
    "BIT_XOR",
    "JSONB_BUILD_OBJECT",
    "JSONB_BUILD_ARRAY",
    "JSONB_OBJECT",
    "JSONB_AGG",
    "JSONB_ARRAY_ELEMENTS",
    "JSONB_ARRAY_ELEMENTS_TEXT",
    "JSONB_EACH",
    "JSONB_EACH_TEXT",
    "JSONB_EXTRACT_PATH",
    "JSONB_EXTRACT_PATH_TEXT",
    "JSONB_TYPEOF",
    "JSONB_STRIP_NULLS",
    "JSONB_SET",
    "JSONB_INSERT",
    "JSONB_PRETTY",
    "JSONB_ARRAY_LENGTH",
    "JSONB_OBJECT_KEYS",
    "JSONB_PATH_EXISTS",
    "JSONB_PATH_QUERY",
    "JSONB_PATH_QUERY_ARRAY",
    "JSONB_PATH_QUERY_FIRST",
    "JSON_BUILD_OBJECT",
    "JSON_BUILD_ARRAY",
    "JSON_AGG",
    "JSON_ARRAY_ELEMENTS",
    "JSON_ARRAY_ELEMENTS_TEXT",
    "JSON_EACH",
    "JSON_EACH_TEXT",
    "JSON_EXTRACT_PATH",
    "JSON_EXTRACT_PATH_TEXT",
    "JSON_TYPEOF",
    "JSON_STRIP_NULLS",
    "JSON_ARRAY_LENGTH",
    "JSON_OBJECT_KEYS",
    "JSON_POPULATE_RECORD",
    "JSON_POPULATE_RECORDSET",
    "JSON_TO_RECORD",
    "JSON_TO_RECORDSET",
    "ROW_TO_JSON",
    "TO_JSON",
    "TO_JSONB",
    "GREATEST",
    "LEAST",
    "CEIL",
    "CEILING",
    "FLOOR",
    "SIGN",
    "MOD",
    "POWER",
    "SQRT",
    "CBRT",
    "LOG",
    "LN",
    "EXP",
    "PI",
    "DEGREES",
    "RADIANS",
    "TRUNC",
    "WIDTH_BUCKET",
    "DIV",
    "GCD",
    "LCM",
    "FACTORIAL",
    "PG_TYPEOF",
    "PG_COLUMN_SIZE",
    "PG_SIZE_PRETTY",
    "PG_TOTAL_RELATION_SIZE",
    "PG_RELATION_SIZE",
    "PG_DATABASE_SIZE",
    "PG_TABLESPACE_SIZE",
    "CURRENT_SCHEMA",
    "CURRENT_SCHEMAS",
    "CURRENT_DATABASE",
    "CURRENT_USER",
    "CURRENT_ROLE",
    "SESSION_USER",
    "INET_ATON",
    "INET_NTOA",
    "HOST",
    "HOSTMASK",
    "MASKLEN",
    "NETMASK",
    "NETWORK",
    "SET_MASKLEN",
    "TEXT",
    "ABBREV",
    "BROADCAST",
    "FAMILY",
    "GEN_RANDOM_UUID",
    "UUID_GENERATE_V4",
    "TXID_CURRENT",
    "TXID_CURRENT_SNAPSHOT",
];

const SQL_TYPES: &[&str] = &[
    // SQLite / standard
    "INTEGER",
    "INT",
    "TINYINT",
    "SMALLINT",
    "MEDIUMINT",
    "BIGINT",
    "UNSIGNED",
    "INT2",
    "INT8",
    "TEXT",
    "CLOB",
    "BLOB",
    "REAL",
    "DOUBLE",
    "FLOAT",
    "NUMERIC",
    "DECIMAL",
    "BOOLEAN",
    "DATE",
    "DATETIME",
    "VARCHAR",
    "CHAR",
    "NCHAR",
    "NVARCHAR",
    // PostgreSQL
    "SERIAL",
    "BIGSERIAL",
    "SMALLSERIAL",
    "BOOL",
    "INT4",
    "FLOAT4",
    "FLOAT8",
    "TIMESTAMP",
    "TIMESTAMPTZ",
    "INTERVAL",
    "TIMETZ",
    "UUID",
    "JSON",
    "JSONB",
    "XML",
    "BYTEA",
    "CIDR",
    "INET",
    "MACADDR",
    "MACADDR8",
    "MONEY",
    "BIT",
    "VARBIT",
    "POINT",
    "LINE",
    "LSEG",
    "BOX",
    "PATH",
    "POLYGON",
    "CIRCLE",
    "TSQUERY",
    "TSVECTOR",
    "OID",
    "REGCLASS",
    "REGTYPE",
    "REGPROC",
    "RECORD",
    "VOID",
    "ARRAY",
    "HSTORE",
    "LTREE",
    "INT4RANGE",
    "INT8RANGE",
    "NUMRANGE",
    "TSRANGE",
    "TSTZRANGE",
    "DATERANGE",
    "INT4MULTIRANGE",
    "INT8MULTIRANGE",
    "NUMMULTIRANGE",
    "TSMULTIRANGE",
    "TSTZMULTIRANGE",
    "DATEMULTIRANGE",
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

            // PostgreSQL dollar-quoted strings ($$...$$, $tag$...$tag$)
            if ch == '$'
                && i + 1 < chars.len()
                && (chars[i + 1] == '$' || chars[i + 1].is_alphabetic() || chars[i + 1] == '_')
            {
                // Find the tag: $$ or $tag$
                let start = i;
                i += 1;
                if chars[i] != '$' {
                    while i < chars.len() && chars[i] != '$' {
                        if !chars[i].is_alphanumeric() && chars[i] != '_' {
                            break;
                        }
                        i += 1;
                    }
                }
                if i < chars.len() && chars[i] == '$' {
                    i += 1;
                    let tag: String = chars[start..i].iter().collect();
                    let tag_len = tag.len();
                    // Find closing tag
                    let mut found = false;
                    while i + tag_len <= chars.len() {
                        let candidate: String = chars[i..i + tag_len].iter().collect();
                        if candidate == tag {
                            i += tag_len;
                            found = true;
                            break;
                        }
                        i += 1;
                    }
                    if !found {
                        i = chars.len();
                    }
                    tokens.push(Token {
                        text: chars[start..i].iter().collect(),
                        token_type: TokenType::String,
                    });
                    continue;
                }
                // Not a valid dollar-quote, treat $ as operator
                i = start;
                i += 1;
                tokens.push(Token {
                    text: "$".to_string(),
                    token_type: TokenType::Operator,
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
            if ch.is_ascii_digit()
                || (ch == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
            {
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
                | '!' | '&' | '|' | '^' | '~' | ':' | '@' | '?' | '#' => {
                    // Check for multi-char operators
                    if i + 1 < chars.len() {
                        let next = chars[i + 1];
                        match (ch, next) {
                            ('<', '=')
                            | ('>', '=')
                            | ('!', '=')
                            | ('<', '>')
                            | ('|', '|')
                            | ('<', '<')
                            | ('>', '>')
                            | (':', ':')  // PostgreSQL cast
                            | ('@', '>')  // PostgreSQL contains
                            | ('<', '@')  // PostgreSQL contained by
                            | ('?', '|')  // PostgreSQL jsonb any key
                            | ('?', '&')  // PostgreSQL jsonb all keys
                            => {
                                i += 2;
                                format!("{}{}", ch, next)
                            }
                            ('#', '>') => {
                                // Could be #> or #>>
                                if i + 2 < chars.len() && chars[i + 2] == '>' {
                                    i += 3;
                                    "#>>".to_string()
                                } else {
                                    i += 2;
                                    "#>".to_string()
                                }
                            }
                            ('-', '>') => {
                                // Could be -> or ->>
                                if i + 2 < chars.len() && chars[i + 2] == '>' {
                                    i += 3;
                                    "->>".to_string()
                                } else {
                                    i += 2;
                                    "->".to_string()
                                }
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
