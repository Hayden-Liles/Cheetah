use crate::ast::Expr;

/// Represents the context in which parsing is occurring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserContext {
    /// Normal parsing context
    Normal,
    
    /// Inside a function
    Function,
    
    /// Inside a loop
    Loop,
    
    /// Inside a comprehension
    Comprehension,
    
    /// Inside a match statement
    Match,
}

/// Function parameter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterKind {
    /// Normal parameter
    #[allow(dead_code)]
    Normal,
    
    /// Position-only parameter (before /)
    #[allow(dead_code)]
    PositionalOnly,
    
    /// Variadic positional parameter (*args)
    #[allow(dead_code)]
    VarArgs,
    
    /// Keyword-only parameter (after *)
    #[allow(dead_code)]
    KeywordOnly,
    
    /// Variadic keyword parameter (**kwargs)
    #[allow(dead_code)]
    KwArgs,
}

/// Source code position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
}

impl SourcePos {
    /// Create a new source position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
    
    /// Create a source position from a token
    #[allow(dead_code)]
    pub fn from_token(token: &crate::lexer::Token) -> Self {
        Self::new(token.line, token.column)
    }
}

/// Extension trait for getting the source position of an AST node
pub trait GetSourcePos {
    /// Get the source position of this node
    #[allow(dead_code)]
    fn get_source_pos(&self) -> SourcePos;
}

impl GetSourcePos for Expr {
    fn get_source_pos(&self) -> SourcePos {
        SourcePos::new(self.get_line(), self.get_column())
    }
}

/// Extension trait for getting line and column information
pub trait GetLocation {
    /// Get the line number of this node
    fn get_line(&self) -> usize;
    
    /// Get the column number of this node
    fn get_column(&self) -> usize;
}

impl GetLocation for Expr {
    fn get_line(&self) -> usize {
        match self {
            Expr::BoolOp { line, .. } => *line,
            Expr::BinOp { line, .. } => *line,
            Expr::UnaryOp { line, .. } => *line,
            Expr::Lambda { line, .. } => *line,
            Expr::IfExp { line, .. } => *line,
            Expr::Dict { line, .. } => *line,
            Expr::Set { line, .. } => *line,
            Expr::ListComp { line, .. } => *line,
            Expr::SetComp { line, .. } => *line,
            Expr::DictComp { line, .. } => *line,
            Expr::GeneratorExp { line, .. } => *line,
            Expr::Await { line, .. } => *line,
            Expr::Yield { line, .. } => *line,
            Expr::YieldFrom { line, .. } => *line,
            Expr::Compare { line, .. } => *line,
            Expr::Call { line, .. } => *line,
            Expr::Num { line, .. } => *line,
            Expr::Str { line, .. } => *line,
            Expr::FormattedValue { line, .. } => *line,
            Expr::JoinedStr { line, .. } => *line,
            Expr::Bytes { line, .. } => *line,
            Expr::NameConstant { line, .. } => *line,
            Expr::Ellipsis { line, .. } => *line,
            Expr::Constant { line, .. } => *line,
            Expr::Attribute { line, .. } => *line,
            Expr::Subscript { line, .. } => *line,
            Expr::Starred { line, .. } => *line,
            Expr::Name { line, .. } => *line,
            Expr::List { line, .. } => *line,
            Expr::Tuple { line, .. } => *line,
            Expr::NamedExpr { line, .. } => *line,
        }
    }

    fn get_column(&self) -> usize {
        match self {
            Expr::BoolOp { column, .. } => *column,
            Expr::BinOp { column, .. } => *column,
            Expr::UnaryOp { column, .. } => *column,
            Expr::Lambda { column, .. } => *column,
            Expr::IfExp { column, .. } => *column,
            Expr::Dict { column, .. } => *column,
            Expr::Set { column, .. } => *column,
            Expr::ListComp { column, .. } => *column,
            Expr::SetComp { column, .. } => *column,
            Expr::DictComp { column, .. } => *column,
            Expr::GeneratorExp { column, .. } => *column,
            Expr::Await { column, .. } => *column,
            Expr::Yield { column, .. } => *column,
            Expr::YieldFrom { column, .. } => *column,
            Expr::Compare { column, .. } => *column,
            Expr::Call { column, .. } => *column,
            Expr::Num { column, .. } => *column,
            Expr::Str { column, .. } => *column,
            Expr::FormattedValue { column, .. } => *column,
            Expr::JoinedStr { column, .. } => *column,
            Expr::Bytes { column, .. } => *column,
            Expr::NameConstant { column, .. } => *column,
            Expr::Ellipsis { column, .. } => *column,
            Expr::Constant { column, .. } => *column,
            Expr::Attribute { column, .. } => *column,
            Expr::Subscript { column, .. } => *column,
            Expr::Starred { column, .. } => *column,
            Expr::Name { column, .. } => *column,
            Expr::List { column, .. } => *column,
            Expr::Tuple { column, .. } => *column,
            Expr::NamedExpr { column, .. } => *column,
        }
    }
}

/// Represents an AST node with source location information
#[derive(Debug, Clone)]
pub struct Located<T> {
    /// The actual node
    #[allow(dead_code)]
    pub node: T,
    /// The source position of the node
    pub pos: SourcePos,
}

impl<T> Located<T> {
    /// Create a new located node
    #[allow(dead_code)]
    pub fn new(node: T, line: usize, column: usize) -> Self {
        Self {
            node,
            pos: SourcePos::new(line, column),
        }
    }
    
    /// Create a new located node from a source position
    #[allow(dead_code)]
    pub fn with_pos(node: T, pos: SourcePos) -> Self {
        Self { node, pos }
    }
    
    /// Map the inner value while preserving the location
    #[allow(dead_code)]
    pub fn map<U, F>(self, f: F) -> Located<U>
    where
        F: FnOnce(T) -> U,
    {
        Located {
            node: f(self.node),
            pos: self.pos,
        }
    }
}

impl<T> GetSourcePos for Located<T> {
    fn get_source_pos(&self) -> SourcePos {
        self.pos
    }
}

/// Represents the associativity of an operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Associativity {
    /// Left-to-right associativity (e.g., +, -, *)
    Left,
    /// Right-to-left associativity (e.g., **)
    Right,
    /// Non-associative (e.g., comparison operators)
    None,
}