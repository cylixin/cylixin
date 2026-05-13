// Type annotations used in variable declarations and function signatures
#[derive(Debug, Clone, PartialEq)]
pub enum CyType {
    Int,
    Long,
    Float,
    StringType,
    Char,
    Bool,
    Arr(Option<Box<CyType>>),
    Set(Option<Box<CyType>>),
    Dic(Option<Box<CyType>>, Option<Box<CyType>>),
    Null,
    Unknown(String), // unresolved - semantic pass will deal with this
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, StrictEq, NotEq,
    Lt, Gt, LtEq, GtEq,
    And, Or,
    BitAnd, BitOr, Shl, Shr,  // TODO: implement in interpreter
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    PowAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLit(i64),
    LongLit(i64),
    FloatLit(f64),
    StringLit(String),
    CharLit(char),
    BoolLit(bool),
    NullLit,

    Ident(String),

    BinaryOp {
        left:  Box<Expr>,
        op:    BinaryOp,
        right: Box<Expr>,
    },

    UnaryOp {
        op:   UnaryOp,
        expr: Box<Expr>,
    },

    // all function calls are @prefixed in source, but by the time
    // we're here the @ is gone and we just have the name + args
    Call {
        name: String,
        args: Vec<Expr>,
    },

    Index {
        collection: Box<Expr>,
        index:      Box<Expr>,
    },

    ArrayLit(Vec<Expr>),
    Grouped(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl {
        kind:        VarKind,
        name:        String,
        type_ann:    Option<CyType>,
        initialiser: Option<Expr>,
    },

    Assign {
        target: AssignTarget,
        op:     AssignOp,
        value:  Expr,
    },

    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        elif_arms: Vec<ElifArm>,
        else_body: Option<Vec<Stmt>>,
        end_when:  Option<EndWhen>,
    },

    ForRange {
        var:      String,
        from:     Expr,
        to:       Expr,
        body:     Vec<Stmt>,
        label:    Option<String>,
        end_when: Option<EndWhen>,
    },

    ForC {
        init:     Box<Stmt>,
        cond:     Expr,
        update:   Box<Stmt>,
        body:     Vec<Stmt>,
        label:    Option<String>,
        end_when: Option<EndWhen>,
    },

    While {
        condition: Expr,
        body:      Vec<Stmt>,
        label:     Option<String>,
        end_when:  Option<EndWhen>,
    },

    FunDecl {
        name:        String,
        params:      Vec<Param>,
        return_type: Option<CyType>,
        body:        Vec<Stmt>,
    },

    Return(Option<Expr>),
    Break(Option<String>),
    ExprStmt(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarKind { Let, Var, Const }

#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Ident(String),
    Index { name: String, index: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElifArm {
    pub condition: Expr,
    pub body:      Vec<Stmt>,
}

// `endif when (x < 0): default_val` - lets a block bail early with a value
#[derive(Debug, Clone, PartialEq)]
pub struct EndWhen {
    pub condition: Expr,
    pub value:     Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name:     String,
    pub type_ann: CyType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
}

impl Program {
    pub fn new(body: Vec<Stmt>) -> Self {
        Program { body }
    }
}
