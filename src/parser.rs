use crate::tokeniser::{Keyword, Location, Token, TokenWithLocation, Tokeniser, TokeniserError};

#[derive(PartialEq, Debug)]
pub enum Statement {
    Select(Select),
    Insert(Insert),
    Update(Update),
    Delete(Delete),
    Create(Create),
}

#[derive(PartialEq, Debug)]
enum Value {
    Number(String),
    String(String),
    Bool(bool),
    Null,
}

#[derive(PartialEq, Debug)]
enum Op {
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(PartialEq, Debug)]
enum Expr {
    Ident(String),
    CompoundIdent(Vec<String>),
    Wildcard,
    QualifiedWildcard(Vec<String>),
    Value(Value),
    IsNull(Box<Expr>),
    IsNotNull(Box<Expr>),
    InList { expr: Box<Expr>, list: Vec<Expr>, negated: bool },
    Between { expr: Box<Expr>, negated: bool, low: Box<Expr>, high: Box<Expr> },
    BinaryOp { left: Box<Expr>, op: Op, right: Box<Expr> },
    // TODO: UnaryOp
    // TODO: functions
    // TODO: subquery
}

#[derive(PartialEq, Debug)]
enum FromTable {
    Table { name: Vec<String>, alias: Option<String> },
    Derived { alias: Option<String>, select: Box<Select> },
}

#[derive(PartialEq, Debug)]
struct OrderByExpr {
    expr: Expr,
    desc: bool, // Default is false/ASC
}

#[derive(PartialEq, Debug)]
enum SelectItem {
    Expr(Expr),
    AliasedExpr { expr: Expr, alias: String },
    QualifiedWildcard(Vec<String>),
    Wildcard,
}

#[derive(PartialEq, Debug)]
pub struct Select {
    projection: Vec<SelectItem>,
    from: FromTable,
    joins: Vec<Select>,
    filter: Option<Expr>,
    group: Vec<Expr>,
    order: OrderByExpr,
    limit: Expr,
}

#[derive(PartialEq, Debug)]
pub struct Insert {}

#[derive(PartialEq, Debug)]
pub struct Update {}

#[derive(PartialEq, Debug)]
pub struct Delete {}

#[derive(PartialEq, Debug)]
pub struct Create {
    name: String,
    columns: Vec<ColumnDef>,
}

#[derive(PartialEq, Debug)]
enum ColumnType {
    Int,
    Varchar(u16),
}

#[derive(PartialEq, Debug)]
struct ColumnDef {
    ty: ColumnType,
    name: String,
    // TODO: constraints
}

#[derive(Debug)]
pub enum ParserError {
    TokeniserError(String),
    Unexpected(String),
}

struct Unexpected<'a>(&'a Token, &'a Location);

impl<'a> std::fmt::Display for Unexpected<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: unexpected token {:?}", self.1, self.0)
    }
}

impl<'a> From<Unexpected<'a>> for ParserError {
    fn from(value: Unexpected<'a>) -> Self {
        Self::Unexpected(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ParserError>;

pub struct Parser {
    tokens: Vec<TokenWithLocation>,
    index: usize,
}

impl Parser {
    pub fn new(src: &str) -> Result<Self> {
        Tokeniser::new(src)
            .collect_with_location()
            .map_err(|e| ParserError::TokeniserError(e.to_string()))
            .map(|tokens| Self { tokens, index: 0 })
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        loop {
            statements.push({
                let TokenWithLocation(token, location) = self.peek();
                match token {
                    Token::Keyword(kw) => match kw {
                        Keyword::Select => Statement::Select(self.parse_select()?),
                        Keyword::Insert => Statement::Insert(self.parse_insert()?),
                        Keyword::Update => Statement::Update(self.parse_update()?),
                        Keyword::Delete => Statement::Delete(self.parse_delete()?),
                        Keyword::Create => Statement::Create(self.parse_create()?),
                        _ => Err(Unexpected(&token, &location))?,
                    },
                    Token::Semicolon => continue,
                    Token::Eof => break,
                    _ => Err(Unexpected(&token, &location))?,
                }
            });
        }

        Ok(statements)
    }

    fn parse_select(&mut self) -> Result<Select> {
        self.parse_keywords(&[Keyword::Select])?;

        let projection = self.parse_projection();

        self.parse_keywords(&[Keyword::From])?;

        // parse table and joins

        if self.check_keywords(&[Keyword::Where]) {
            // parse filter
        };

        if self.check_keywords(&[Keyword::Group, Keyword::By]) {
            // parse group
        }

        if self.check_keywords(&[Keyword::Order, Keyword::By]) {
            // parse order
        }

        if self.check_keywords(&[Keyword::Limit]) {
            // parse limit
        }

        Ok(Select {
            projection: todo!(),
            from: todo!(),
            joins: todo!(),
            filter: todo!(),
            group: todo!(),
            order: todo!(),
            limit: todo!(),
        })
    }

    fn parse_insert(&mut self) -> Result<Insert> {
        Ok(Insert {})
    }

    fn parse_update(&mut self) -> Result<Update> {
        Ok(Update {})
    }

    fn parse_delete(&mut self) -> Result<Delete> {
        Ok(Delete {})
    }

    fn parse_create(&mut self) -> Result<Create> {
        self.parse_keywords(&[Keyword::Create, Keyword::Table])?;

        let TokenWithLocation(token, location) = self.next();
        let name = match token {
            Token::Ident(name) => name,
            _ => Err(Unexpected(&token, &location))?,
        };

        self.parse_tokens(&[Token::LParen])?;
        let mut columns = Vec::new();
        while {
            columns.push(self.parse_column_def()?);
            self.check_tokens(&[Token::Comma])
        } {}
        self.parse_tokens(&[Token::RParen])?;

        Ok(Create { name, columns })
    }

    fn parse_column_def(&mut self) -> Result<ColumnDef> {
        let TokenWithLocation(token, location) = self.next();
        let name = match token {
            Token::Ident(name) => name,
            _ => Err(Unexpected(&token, &location))?,
        };

        let TokenWithLocation(token, location) = self.next();
        let ty = match token {
            Token::Keyword(Keyword::Int) => ColumnType::Int,
            Token::Keyword(Keyword::Varchar) => {
                self.parse_tokens(&[Token::LParen])?;
                let TokenWithLocation(token, location) = self.next();
                let max = match token {
                    Token::NumberLiteral(ref max) => {
                        max.parse().map_err(|_| Unexpected(&token, &location))?
                    }
                    _ => Err(Unexpected(&token, &location))?,
                };
                self.parse_tokens(&[Token::RParen])?;
                ColumnType::Varchar(max)
            }
            _ => Err(Unexpected(&token, &location))?,
        };

        Ok(ColumnDef { ty, name })
    }

    fn parse_projection(&mut self) -> Result<Vec<SelectItem>> {
        let mut items = Vec::new();
        while {
            items.push(self.parse_select_item()?);
            self.check_tokens(&[Token::Comma])
        } {}

        Ok(items)
    }

    fn parse_select_item(&mut self) -> Result<SelectItem> {
        // Try parse wildcard or qualified wildcard
        // Parse expr and optional alias
        let index = self.index;
        let TokenWithLocation(token, _) = self.next();
        match token {
            Token::Asterisk => return Ok(SelectItem::Wildcard),
            Token::Ident(a) => {
                // Try to parse a qualified ident, else reset index and parse_expr
                let mut parts = Vec::with_capacity(2);
                if self.check_tokens(&[Token::Dot]) {
                    parts.push(a);

                    let TokenWithLocation(b, location) = self.next();
                    match b {
                        Token::Ident(b) => parts.push(b),
                        Token::Asterisk => return Ok(SelectItem::QualifiedWildcard(parts)),
                        _ => Err(Unexpected(&b, &location))?,
                    };

                    if self.check_tokens(&[Token::Dot]) {
                        let TokenWithLocation(c, location) = self.next();
                        match c {
                            Token::Ident(_) => {}
                            Token::Asterisk => return Ok(SelectItem::QualifiedWildcard(parts)),
                            _ => Err(Unexpected(&c, &location))?,
                        };
                    }
                }
            }
            _ => {}
        };

        self.index = index;
        let expr = self.parse_expr(0)?;
        if self.check_keywords(&[Keyword::As]) {
            let TokenWithLocation(token, location) = self.next();
            match token {
                Token::Ident(alias) => return Ok(SelectItem::AliasedExpr { expr, alias }),
                _ => Err(Unexpected(&token, &location))?,
            };
        };

        Ok(SelectItem::Expr(expr))
    }

    fn parse_expr(&mut self, prec: u8) -> Result<Expr> {
        let mut expr = self.parse_prefix()?;
        loop {
            let next_prec = self.next_prec()?;
            if prec >= next_prec {
                break;
            }
            expr = self.parse_infix(expr, next_prec)?;
        }

        Ok(expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        let TokenWithLocation(token, location) = self.peek();
        let expr = match token {
            Token::Keyword(Keyword::False)
            | Token::Keyword(Keyword::True)
            | Token::Keyword(Keyword::Null)
            | Token::StringLiteral(_)
            | Token::NumberLiteral(_) => Expr::Value(self.parse_value()?),

            Token::Ident(a) => {
                self.next();

                let mut parts = Vec::with_capacity(2);
                if self.check_tokens(&[Token::Dot]) {
                    parts.push(a);

                    let TokenWithLocation(b, location) = self.next();
                    match b {
                        Token::Ident(b) => parts.push(b),
                        _ => Err(Unexpected(&b, &location))?,
                    };

                    if self.check_tokens(&[Token::Dot]) {
                        let TokenWithLocation(c, location) = self.next();
                        match c {
                            Token::Ident(c) => parts.push(c),
                            _ => Err(Unexpected(&c, &location))?,
                        };
                    }

                    Expr::CompoundIdent(parts)
                } else {
                    Expr::Ident(a)
                }
            }

            Token::LParen => {
                self.next();
                let expr = self.parse_expr(0)?;
                self.parse_tokens(&[Token::RParen])?;
                expr
            }

            _ => Err(Unexpected(&token, &location))?,
        };

        Ok(expr)
    }

    fn parse_infix(&mut self, expr: Expr, prec: u8) -> Result<Expr> {
        let TokenWithLocation(token, location) = self.next();
        let op = match token {
            Token::Keyword(kw) => match kw {
                Keyword::And => Some(Op::And),
                Keyword::Or => Some(Op::Or),
                _ => None,
            },
            Token::Eq => Some(Op::Eq),
            Token::Neq => Some(Op::Neq),
            Token::Lt => Some(Op::Lt),
            Token::Le => Some(Op::Le),
            Token::Gt => Some(Op::Gt),
            Token::Ge => Some(Op::Ge),
            _ => None,
        };

        if let Some(op) = op {
            return Ok(Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(self.parse_expr(prec)?),
            });
        }

        let expr = match token {
            Token::Keyword(kw) => match kw {
                Keyword::Is => {
                    // [not] null, true, false
                    todo!()
                }
                Keyword::Not | Keyword::Between | Keyword::In => {
                    self.index -= 1;
                    let negated = self.check_keywords(&[Keyword::Not]);
                    if self.check_keywords(&[Keyword::Between]) {
                        self.parse_between(expr, negated)?
                    } else if self.check_keywords(&[Keyword::In]) {
                        self.parse_in(expr, negated)?
                    } else {
                        // Should be the next token?
                        Err(Unexpected(&token, &location))?
                    }
                }
                _ => Err(Unexpected(&token, &location))?,
            },
            _ => Err(Unexpected(&token, &location))?,
        };

        Ok(expr)
    }

    fn next_prec(&self) -> Result<u8> {
        let TokenWithLocation(token, _) = self.peek();
        let prec = match token {
            Token::Eq | Token::Neq | Token::Lt | Token::Le | Token::Gt | Token::Ge => 20,
            Token::Keyword(Keyword::And) => 10,
            Token::Keyword(Keyword::Or) => 5,

            Token::Keyword(Keyword::Not) => {
                let TokenWithLocation(token, location) = self.peek_n(1);
                match token {
                    Token::Keyword(Keyword::Between) => 20,
                    Token::Keyword(Keyword::In) => 20,
                    _ => Err(Unexpected(&token, &location))?,
                }
            }
            Token::Keyword(Keyword::Is) => 17,
            Token::Keyword(Keyword::Between) => 20,
            Token::Keyword(Keyword::In) => 20,
            _ => 0,
        };

        Ok(prec)
    }

    fn parse_value(&mut self) -> Result<Value> {
        let TokenWithLocation(token, location) = self.next();
        match token {
            Token::Keyword(Keyword::False) => Ok(Value::Bool(false)),
            Token::Keyword(Keyword::True) => Ok(Value::Bool(true)),
            Token::Keyword(Keyword::Null) => Ok(Value::Null),
            Token::StringLiteral(s) => Ok(Value::String(s)),
            Token::NumberLiteral(n) => Ok(Value::Number(n)),
            _ => Err(Unexpected(&token, &location))?,
        }
    }

    fn parse_between(&mut self, expr: Expr, negated: bool) -> Result<Expr> {
        let low = self.parse_expr(20)?;
        self.parse_keywords(&[Keyword::And])?;
        let high = self.parse_expr(20)?;

        Ok(Expr::Between {
            expr: Box::new(expr),
            negated,
            low: Box::new(low),
            high: Box::new(high),
        })
    }

    fn parse_in(&mut self, expr: Expr, negated: bool) -> Result<Expr> {
        let mut list = Vec::new();

        self.parse_tokens(&[Token::LParen])?;
        while {
            list.push(self.parse_expr(0)?);
            self.check_tokens(&[Token::Comma])
        } {}
        self.parse_tokens(&[Token::RParen])?;

        Ok(Expr::InList { expr: Box::new(expr), list, negated })
    }

    // Will advance and return true if tokens match, otherwise walk back and return false
    fn check_tokens(&mut self, tokens: &[Token]) -> bool {
        let index = self.index;

        for want in tokens {
            match self.peek() {
                TokenWithLocation(ref have, ..) if want == have => {
                    self.next();
                    continue;
                }
                _ => {
                    self.index = index;
                    return false;
                }
            }
        }

        true
    }

    fn check_keywords(&mut self, keywords: &[Keyword]) -> bool {
        let index = self.index;

        for want in keywords {
            match self.peek() {
                TokenWithLocation(Token::Keyword(ref have), ..) if want == have => {
                    self.next();
                    continue;
                }
                _ => {
                    self.index = index;
                    return false;
                }
            }
        }

        true
    }

    fn parse_keywords(&mut self, keywords: &[Keyword]) -> Result<()> {
        for want in keywords {
            let TokenWithLocation(token, location) = self.next();
            match token {
                Token::Keyword(ref have) if want == have => continue,
                _ => Err(Unexpected(&token, &location))?,
            }
        }

        Ok(())
    }

    fn parse_tokens(&mut self, tokens: &[Token]) -> Result<()> {
        for want in tokens {
            let TokenWithLocation(ref have, location) = self.next();
            if want == have {
                continue;
            }

            Err(Unexpected(&have, &location))?;
        }

        Ok(())
    }

    fn next(&mut self) -> TokenWithLocation {
        self.index += 1;
        self.get(self.index - 1)
    }

    fn peek(&self) -> TokenWithLocation {
        self.peek_n(0)
    }

    fn peek_n(&self, n: usize) -> TokenWithLocation {
        self.get(self.index + n)
    }

    fn get(&self, i: usize) -> TokenWithLocation {
        self.tokens
            .get(i)
            .map(|t| t.clone())
            .unwrap_or(TokenWithLocation(Token::Eof, Default::default()))
    }
}

#[cfg(test)]
mod test {
    use super::{ColumnDef, ColumnType, Create, Expr, Op, Parser, SelectItem, Statement, Value};

    #[test]
    fn test_create_statement() {
        let input = "
            CREATE TABLE t1 (
                c1 INT,
                c2 VARCHAR(1024)
            )";

        let want = vec![Statement::Create(Create {
            name: "t1".into(),
            columns: vec![
                ColumnDef { ty: ColumnType::Int, name: "c1".into() },
                ColumnDef { ty: ColumnType::Varchar(1024), name: "c2".into() },
            ],
        })];

        let have = Parser::new(input).unwrap().parse().unwrap();
        assert_eq!(want, have)
    }

    #[test]
    fn test_parse_projection() {
        let input = "t1.*, *, s1.t1.c1";

        let want = vec![
            SelectItem::QualifiedWildcard(vec!["t1".into()]),
            SelectItem::Wildcard,
            SelectItem::Expr(Expr::CompoundIdent(vec!["s1".into(), "t1".into(), "c1".into()])),
        ];
        let have = Parser::new(input).unwrap().parse_projection().unwrap();
        assert_eq!(want, have)
    }

    macro_rules! test_parse_expr {
        ($name:tt, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let mut parser = Parser::new($input).unwrap();
                let have = parser.parse_expr(0).unwrap();
                assert_eq!($want, have);
            }
        };
    }

    test_parse_expr!(
        test_expr_binary_op,
        "c1 < 5",
        Expr::BinaryOp {
            left: Box::new(Expr::Ident("c1".into())),
            op: Op::Lt,
            right: Box::new(Expr::Value(Value::Number("5".into()))),
        }
    );

    test_parse_expr!(
        test_expr_binary_op_in,
        "c1 < 5 and c2 in (1, \"2\", 3, \"4\")",
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Ident("c1".into())),
                op: Op::Lt,
                right: Box::new(Expr::Value(Value::Number("5".into()))),
            }),
            op: Op::And,
            right: Box::new(Expr::InList {
                expr: Box::new(Expr::Ident("c2".into())),
                list: vec![
                    Expr::Value(Value::Number("1".into())),
                    Expr::Value(Value::String("2".into())),
                    Expr::Value(Value::Number("3".into())),
                    Expr::Value(Value::String("4".into())),
                ],
                negated: false,
            }),
        }
    );

    test_parse_expr!(
        test_expr_binary_op_not_in,
        "c1 < 5 and c2 not in (1, \"2\", 3, \"4\")",
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Ident("c1".into())),
                op: Op::Lt,
                right: Box::new(Expr::Value(Value::Number("5".into()))),
            }),
            op: Op::And,
            right: Box::new(Expr::InList {
                expr: Box::new(Expr::Ident("c2".into())),
                list: vec![
                    Expr::Value(Value::Number("1".into())),
                    Expr::Value(Value::String("2".into())),
                    Expr::Value(Value::Number("3".into())),
                    Expr::Value(Value::String("4".into())),
                ],
                negated: true,
            }),
        }
    );

    test_parse_expr!(
        test_expr_binary_op_not_in_parens,
        "(c1 < 5) and (c2 not in (1, \"2\", 3, \"4\"))",
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Ident("c1".into())),
                op: Op::Lt,
                right: Box::new(Expr::Value(Value::Number("5".into()))),
            }),
            op: Op::And,
            right: Box::new(Expr::InList {
                expr: Box::new(Expr::Ident("c2".into())),
                list: vec![
                    Expr::Value(Value::Number("1".into())),
                    Expr::Value(Value::String("2".into())),
                    Expr::Value(Value::Number("3".into())),
                    Expr::Value(Value::String("4".into())),
                ],
                negated: true,
            }),
        }
    );

    test_parse_expr!(
        test_expr_parens,
        "c1 < (5 < c2) AND (c1 < 5) < c2",
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Ident("c1".into())),
                op: Op::Lt,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(Value::Number("5".into()))),
                    op: Op::Lt,
                    right: Box::new(Expr::Ident("c2".into()))
                })
            }),
            op: Op::And,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Ident("c1".into())),
                    op: Op::Lt,
                    right: Box::new(Expr::Value(Value::Number("5".into())))
                }),
                op: Op::Lt,
                right: Box::new(Expr::Ident("c2".into()))
            })
        }
    );

    test_parse_expr!(
        test_expr_between,
        "c1 between 0 and 200",
        Expr::Between {
            expr: Box::new(Expr::Ident("c1".into())),
            negated: false,
            low: Box::new(Expr::Value(Value::Number("0".into()))),
            high: Box::new(Expr::Value(Value::Number("200".into()))),
        }
    );

    test_parse_expr!(
        test_expr_not_between,
        "c1 not between 0 and 200",
        Expr::Between {
            expr: Box::new(Expr::Ident("c1".into())),
            negated: true,
            low: Box::new(Expr::Value(Value::Number("0".into()))),
            high: Box::new(Expr::Value(Value::Number("200".into()))),
        }
    );

    test_parse_expr!(
        test_expr_compound_ident,
        "s1.t1.c1 > 5",
        Expr::BinaryOp {
            left: Box::new(Expr::CompoundIdent(vec!["s1".into(), "t1".into(), "c1".into()])),
            op: Op::Gt,
            right: Box::new(Expr::Value(Value::Number("5".into()))),
        }
    );
}
