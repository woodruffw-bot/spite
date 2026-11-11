//! Structured statement bodies use distinct native parser frames.

use super::{CatchClause, Diagnostic, Kind, Parser, Span, Statement, StatementKind, SwitchClause};

impl Parser {
    #[inline(never)]
    pub(super) fn block_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        let mut body = Vec::new();
        while !self.at("}") {
            if self.current().kind == Kind::Eof {
                return Err(self.error("unterminated block"));
            }
            body.push(self.statement(true)?);
        }
        self.expect("}")?;
        Ok(self.finish_statement(start, StatementKind::Block(body)))
    }

    #[inline(never)]
    pub(super) fn try_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        // ECMA-262 14.15 requires blocks, not arbitrary statements.
        let body = self.required_block()?;
        let handler = if self.at("catch") {
            let start = self.bump().span.start;
            let parameter = if self.eat("(") {
                let parameter = self.binding_pattern()?;
                self.expect(")")?;
                Some(parameter)
            } else {
                None
            };
            let body = self.required_block()?;
            let span = Span::new(start, body.span.end);
            Some(Box::new(CatchClause {
                parameter,
                body,
                span,
            }))
        } else {
            None
        };
        let finalizer = if self.eat("finally") {
            Some(self.required_block()?)
        } else {
            None
        };
        if handler.is_none() && finalizer.is_none() {
            return Err(self.error("expected catch or finally"));
        }
        Ok(self.finish_statement(
            start,
            StatementKind::Try {
                body,
                handler,
                finalizer,
            },
        ))
    }

    #[inline(never)]
    pub(super) fn switch_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        self.expect("(")?;
        let discriminant = self.expression(1)?;
        self.expect(")")?;
        self.expect("{")?;
        let mut clauses = Vec::new();
        let mut has_default = false;
        while !self.at("}") {
            let start = self.current().span.start;
            let test = if self.eat("case") {
                Some(self.expression(1)?)
            } else if self.at("default") {
                if has_default {
                    return Err(self.error("duplicate default clause"));
                }
                self.bump();
                has_default = true;
                None
            } else {
                return Err(self.error("expected case, default, or }"));
            };
            self.expect(":")?;
            let mut statements = Vec::new();
            while !self.at("case") && !self.at("default") && !self.at("}") {
                if self.current().kind == Kind::Eof {
                    return Err(self.error("unterminated switch"));
                }
                statements.push(self.statement(true)?);
            }
            let end = self.tokens[self.index - 1].span.end;
            clauses.push(SwitchClause {
                test,
                statements,
                span: Span::new(start, end),
            });
        }
        self.expect("}")?;
        Ok(self.finish_statement(
            start,
            StatementKind::Switch {
                discriminant,
                clauses,
            },
        ))
    }

    #[inline(never)]
    pub(super) fn if_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        self.expect("(")?;
        let test = self.expression(1)?;
        self.expect(")")?;
        let consequent = Box::new(self.statement(false)?);
        let alternate = if self.eat("else") {
            Some(Box::new(self.statement(false)?))
        } else {
            None
        };
        Ok(self.finish_statement(
            start,
            StatementKind::If {
                test,
                consequent,
                alternate,
            },
        ))
    }

    #[inline(never)]
    pub(super) fn while_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        self.expect("(")?;
        let test = self.expression(1)?;
        self.expect(")")?;
        let body = Box::new(self.statement(false)?);
        Ok(self.finish_statement(start, StatementKind::While { test, body }))
    }

    #[inline(never)]
    pub(super) fn with_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        self.expect("(")?;
        let object = self.expression_with_in(1, true)?;
        self.expect(")")?;
        let body = Box::new(self.statement(false)?);
        Ok(self.finish_statement(start, StatementKind::With { object, body }))
    }

    #[inline(never)]
    pub(super) fn do_while_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        let body = Box::new(self.statement(false)?);
        self.expect("while")?;
        self.expect("(")?;
        let test = self.expression(1)?;
        self.expect(")")?;
        // ECMA-262 12.10.1 allows ASI after the closing parenthesis of
        // do-while even without a line terminator before the next token.
        self.eat(";");
        Ok(self.finish_statement(start, StatementKind::DoWhile { body, test }))
    }

    #[inline(never)]
    pub(super) fn return_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        if !self.allow_return {
            return Err(self.error("return outside a function body"));
        }
        self.bump();
        // ECMA-262 14.10: a line terminator ends a bare return.
        let value = if self.current().newline
            || self.at(";")
            || self.at("}")
            || self.current().kind == Kind::Eof
        {
            None
        } else {
            Some(self.expression_with_in(1, true)?)
        };
        self.semicolon()?;
        Ok(self.finish_statement(start, StatementKind::Return(value)))
    }

    #[inline(never)]
    pub(super) fn throw_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        if self.current().newline {
            return Err(self.error("line terminator after throw"));
        }
        let expr = self.expression(1)?;
        self.semicolon()?;
        Ok(self.finish_statement(start, StatementKind::Throw(expr)))
    }

    #[inline(never)]
    pub(super) fn for_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        let header = self.for_header()?;
        let body = Box::new(self.statement(false)?);
        Ok(self.finish_statement(start, header.with_body(body)))
    }

    #[inline(never)]
    pub(super) fn labelled_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        let label = self.label_identifier()?;
        self.expect(":")?;
        // ECMA-262 14.13.1: labelled functions require the excluded Annex B
        // extension even in non-strict code.
        if self.at("function") {
            return Err(self.error("labelled functions are not allowed"));
        }
        let body = Box::new(self.statement(false)?);
        let span = Span::new(start, body.span.end);
        Ok(Statement {
            kind: StatementKind::Labelled { label, body },
            span,
        })
    }

    #[inline(never)]
    pub(super) fn expression_statement(&mut self, start: usize) -> Result<Statement, Diagnostic> {
        let expr = self.expression(1)?;
        self.semicolon()?;
        Ok(self.finish_statement(start, StatementKind::Expression(expr)))
    }
}
