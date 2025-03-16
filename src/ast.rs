use std::fmt;

#[derive(Debug, Clone)]
pub enum Stmt {
    FunctionDef {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Box<Stmt>>,
        decorator_list: Vec<Box<Expr>>,
        returns: Option<Box<Expr>>,
        is_async: bool, // For async functions
        line: usize,
        column: usize,
    },
    ClassDef {
        name: String,
        bases: Vec<Box<Expr>>,
        keywords: Vec<(Option<String>, Box<Expr>)>,
        body: Vec<Box<Stmt>>,
        decorator_list: Vec<Box<Expr>>,
        line: usize,
        column: usize,
    },
    Return {
        value: Option<Box<Expr>>,
        line: usize,
        column: usize,
    },
    Delete {
        targets: Vec<Box<Expr>>,
        line: usize,
        column: usize,
    },
    Assign {
        targets: Vec<Box<Expr>>,
        value: Box<Expr>,
        line: usize,
        column: usize,
    },
    AugAssign {
        target: Box<Expr>,
        op: Operator,
        value: Box<Expr>,
        line: usize,
        column: usize,
    },
    AnnAssign {
        target: Box<Expr>,
        annotation: Box<Expr>,
        value: Option<Box<Expr>>,
        line: usize,
        column: usize,
    },
    For {
        target: Box<Expr>,
        iter: Box<Expr>,
        body: Vec<Box<Stmt>>,
        orelse: Vec<Box<Stmt>>,
        is_async: bool, // For async for loops
        line: usize,
        column: usize,
    },
    While {
        test: Box<Expr>,
        body: Vec<Box<Stmt>>,
        orelse: Vec<Box<Stmt>>,
        line: usize,
        column: usize,
    },
    If {
        test: Box<Expr>,
        body: Vec<Box<Stmt>>,
        orelse: Vec<Box<Stmt>>,
        line: usize,
        column: usize,
    },
    With {
        items: Vec<(Box<Expr>, Option<Box<Expr>>)>,
        body: Vec<Box<Stmt>>,
        is_async: bool, // For async with statements
        line: usize,
        column: usize,
    },
    Raise {
        exc: Option<Box<Expr>>,
        cause: Option<Box<Expr>>,
        line: usize,
        column: usize,
    },
    Try {
        body: Vec<Box<Stmt>>,
        handlers: Vec<ExceptHandler>,
        orelse: Vec<Box<Stmt>>,
        finalbody: Vec<Box<Stmt>>,
        line: usize,
        column: usize,
    },
    Assert {
        test: Box<Expr>,
        msg: Option<Box<Expr>>,
        line: usize,
        column: usize,
    },
    Import {
        names: Vec<Alias>,
        line: usize,
        column: usize,
    },
    ImportFrom {
        module: Option<String>,
        names: Vec<Alias>,
        level: usize,
        line: usize,
        column: usize,
    },
    Global {
        names: Vec<String>,
        line: usize,
        column: usize,
    },
    Nonlocal {
        names: Vec<String>,
        line: usize,
        column: usize,
    },
    Expr {
        value: Box<Expr>,
        line: usize,
        column: usize,
    },
    Pass {
        line: usize,
        column: usize,
    },
    Break {
        line: usize,
        column: usize,
    },
    Continue {
        line: usize,
        column: usize,
    },
    Match {
        subject: Box<Expr>,
        cases: Vec<(Box<Expr>, Option<Box<Expr>>, Vec<Box<Stmt>>)>,
        line: usize,
        column: usize,
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    BoolOp {
        op: BoolOperator,
        values: Vec<Box<Expr>>,
        line: usize,
        column: usize,
    },
    BinOp {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
        line: usize,
        column: usize,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expr>,
        line: usize,
        column: usize,
    },
    Lambda {
        args: Vec<Parameter>,
        body: Box<Expr>,
        line: usize,
        column: usize,
    },
    IfExp {
        test: Box<Expr>,
        body: Box<Expr>,
        orelse: Box<Expr>,
        line: usize,
        column: usize,
    },
    Dict {
        keys: Vec<Option<Box<Expr>>>,
        values: Vec<Box<Expr>>,
        line: usize,
        column: usize,
    },
    Set {
        elts: Vec<Box<Expr>>,
        line: usize,
        column: usize,
    },
    ListComp {
        elt: Box<Expr>,
        generators: Vec<Comprehension>,
        line: usize,
        column: usize,
    },
    SetComp {
        elt: Box<Expr>,
        generators: Vec<Comprehension>,
        line: usize,
        column: usize,
    },
    DictComp {
        key: Box<Expr>,
        value: Box<Expr>,
        generators: Vec<Comprehension>,
        line: usize,
        column: usize,
    },
    GeneratorExp {
        elt: Box<Expr>,
        generators: Vec<Comprehension>,
        line: usize,
        column: usize,
    },
    Await {
        value: Box<Expr>,
        line: usize,
        column: usize,
    },
    Yield {
        value: Option<Box<Expr>>,
        line: usize,
        column: usize,
    },
    YieldFrom {
        value: Box<Expr>,
        line: usize,
        column: usize,
    },
    Compare {
        left: Box<Expr>,
        ops: Vec<CmpOperator>,
        comparators: Vec<Box<Expr>>,
        line: usize,
        column: usize,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Box<Expr>>,
        keywords: Vec<(Option<String>, Box<Expr>)>,
        line: usize,
        column: usize,
    },
    Num {
        value: Number,
        line: usize,
        column: usize,
    },
    Str {
        value: String,
        line: usize,
        column: usize,
    },
    FormattedValue {
        value: Box<Expr>,
        conversion: char,
        format_spec: Option<Box<Expr>>,
        line: usize,
        column: usize,
    },
    JoinedStr {
        values: Vec<Box<Expr>>,
        line: usize,
        column: usize,
    },
    Bytes {
        value: Vec<u8>,
        line: usize,
        column: usize,
    },
    NameConstant {
        value: NameConstant,
        line: usize,
        column: usize,
    },
    Ellipsis {
        line: usize,
        column: usize,
    },
    Constant {
        value: Constant,
        line: usize,
        column: usize,
    },
    Attribute {
        value: Box<Expr>,
        attr: String,
        ctx: ExprContext,
        line: usize,
        column: usize,
    },
    Subscript {
        value: Box<Expr>,
        slice: Box<Expr>,
        ctx: ExprContext,
        line: usize,
        column: usize,
    },
    Starred {
        value: Box<Expr>,
        ctx: ExprContext,
        line: usize,
        column: usize,
    },
    Name {
        id: String,
        ctx: ExprContext,
        line: usize,
        column: usize,
    },
    List {
        elts: Vec<Box<Expr>>,
        ctx: ExprContext,
        line: usize,
        column: usize,
    },
    Tuple {
        elts: Vec<Box<Expr>>,
        ctx: ExprContext,
        line: usize,
        column: usize,
    },
    NamedExpr {
        target: Box<Expr>,
        value: Box<Expr>,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprContext {
    Load,
    Store,
    Del,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mult,
    MatMult,
    Div,
    FloorDiv,
    Mod,
    Pow,
    LShift,
    RShift,
    BitOr,
    BitXor,
    BitAnd,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Invert,
    Not,
    UAdd,
    USub,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOperator {
    Eq,
    NotEq,
    Lt,
    LtE,
    Gt,
    GtE,
    Is,
    IsNot,
    In,
    NotIn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
    Complex { real: f64, imag: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum NameConstant {
    None,
    True,
    False,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Num(Number),
    Str(String),
    Bytes(Vec<u8>),
    NameConstant(NameConstant),
    Ellipsis,
}

#[derive(Debug, Clone)]
pub struct Comprehension {
    pub target: Box<Expr>,
    pub iter: Box<Expr>,
    pub ifs: Vec<Box<Expr>>,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct ExceptHandler {
    pub typ: Option<Box<Expr>>,
    pub name: Option<String>,
    pub body: Vec<Box<Stmt>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub asname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub typ: Option<Box<Expr>>,
    pub default: Option<Box<Expr>>,
    pub is_vararg: bool, // For *args
    pub is_kwarg: bool,  // For **kwargs
}

#[derive(Debug, Clone)]
pub struct Module {
    pub body: Vec<Box<Stmt>>,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Module:")?;
        for stmt in &self.body {
            write!(f, "  {}", stmt)?;
        }
        Ok(())
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::FunctionDef { name, .. } => write!(f, "FunctionDef: {}", name),
            Stmt::ClassDef { name, .. } => write!(f, "ClassDef: {}", name),
            Stmt::Return { .. } => write!(f, "Return"),
            Stmt::Delete { .. } => write!(f, "Delete"),
            Stmt::Assign { .. } => write!(f, "Assign"),
            Stmt::AugAssign { .. } => write!(f, "AugAssign"),
            Stmt::AnnAssign { .. } => write!(f, "AnnAssign"),
            Stmt::For { .. } => write!(f, "For"),
            Stmt::While { .. } => write!(f, "While"),
            Stmt::If { .. } => write!(f, "If"),
            Stmt::With { .. } => write!(f, "With"),
            Stmt::Raise { .. } => write!(f, "Raise"),
            Stmt::Try { .. } => write!(f, "Try"),
            Stmt::Assert { .. } => write!(f, "Assert"),
            Stmt::Import { .. } => write!(f, "Import"),
            Stmt::ImportFrom { .. } => write!(f, "ImportFrom"),
            Stmt::Global { .. } => write!(f, "Global"),
            Stmt::Nonlocal { .. } => write!(f, "Nonlocal"),
            Stmt::Expr { .. } => write!(f, "Expr"),
            Stmt::Pass { .. } => write!(f, "Pass"),
            Stmt::Break { .. } => write!(f, "Break"),
            Stmt::Continue { .. } => write!(f, "Continue"),
            Stmt::Match { .. } => write!(f, "Match"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::BoolOp { .. } => write!(f, "BoolOp"),
            Expr::BinOp { .. } => write!(f, "BinOp"),
            Expr::UnaryOp { .. } => write!(f, "UnaryOp"),
            Expr::Lambda { .. } => write!(f, "Lambda"),
            Expr::IfExp { .. } => write!(f, "IfExp"),
            Expr::Dict { .. } => write!(f, "Dict"),
            Expr::Set { .. } => write!(f, "Set"),
            Expr::ListComp { .. } => write!(f, "ListComp"),
            Expr::SetComp { .. } => write!(f, "SetComp"),
            Expr::DictComp { .. } => write!(f, "DictComp"),
            Expr::GeneratorExp { .. } => write!(f, "GeneratorExp"),
            Expr::Await { .. } => write!(f, "Await"),
            Expr::Yield { .. } => write!(f, "Yield"),
            Expr::YieldFrom { .. } => write!(f, "YieldFrom"),
            Expr::Compare { .. } => write!(f, "Compare"),
            Expr::Call { .. } => write!(f, "Call"),
            Expr::Num { value, .. } => write!(f, "Num({:?})", value),
            Expr::Str { value, .. } => write!(f, "Str({})", value),
            Expr::FormattedValue { .. } => write!(f, "FormattedValue"),
            Expr::JoinedStr { .. } => write!(f, "JoinedStr"),
            Expr::Bytes { .. } => write!(f, "Bytes"),
            Expr::NameConstant { value, .. } => write!(f, "NameConstant({:?})", value),
            Expr::Ellipsis { .. } => write!(f, "Ellipsis"),
            Expr::Constant { value, .. } => write!(f, "Constant({:?})", value),
            Expr::Attribute { value, attr, .. } => write!(f, "Attribute({}.{})", value, attr),
            Expr::Subscript { .. } => write!(f, "Subscript"),
            Expr::Starred { .. } => write!(f, "Starred"),
            Expr::Name { id, .. } => write!(f, "Name({})", id),
            Expr::List { .. } => write!(f, "List"),
            Expr::Tuple { .. } => write!(f, "Tuple"),
            Expr::NamedExpr { target, value, .. } => {
                write!(f, "NamedExpr({} := {})", target, value)
            }
        }
    }
}
