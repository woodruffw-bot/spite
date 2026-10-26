//! Typed paths into the immutable async source body.

use crate::{
    Error,
    object::async_execution::{StatementChild, StatementLocation},
};
use spite_core::Span;
use spite_parser::ast::{ArrowBody, Statement, StatementKind};

pub(super) fn active_statement<'a>(
    body: &'a ArrowBody,
    location: Option<&StatementLocation>,
) -> &'a Statement {
    let ArrowBody::Block(body) = body else {
        unreachable!("block statement continuation");
    };
    let location = location.expect("active source statement");
    let mut statement = &body.statements()[location.root];
    for child in &location.children {
        statement = match (child, &statement.kind) {
            (StatementChild::Block(index), StatementKind::Block(body)) => &body[*index],
            (
                StatementChild::SwitchBody { clause, statement },
                StatementKind::Switch { clauses, .. },
            ) => &clauses[*clause].statements[*statement],
            (StatementChild::IfConsequent, StatementKind::If { consequent, .. }) => consequent,
            (
                StatementChild::IfAlternate,
                StatementKind::If {
                    alternate: Some(alternate),
                    ..
                },
            ) => alternate,
            (StatementChild::TryBody, StatementKind::Try { body, .. }) => body,
            (
                StatementChild::CatchBody,
                StatementKind::Try {
                    handler: Some(handler),
                    ..
                },
            ) => &handler.body,
            (
                StatementChild::FinallyBody,
                StatementKind::Try {
                    finalizer: Some(finalizer),
                    ..
                },
            ) => finalizer,
            (
                StatementChild::LoopBody,
                StatementKind::While { body, .. } | StatementKind::DoWhile { body, .. },
            ) => body,
            (StatementChild::ForBody, StatementKind::For { body, .. }) => body,
            (StatementChild::ForOfBody, StatementKind::ForOf { body, .. }) => body,
            (StatementChild::ForInBody, StatementKind::ForIn { body, .. }) => body,
            (StatementChild::LabelBody, StatementKind::Labelled { body, .. }) => body,
            _ => unreachable!("typed statement child path"),
        };
    }
    statement
}

pub(super) fn child_location(
    parent: &StatementLocation,
    child: StatementChild,
    span: Span,
) -> Result<StatementLocation, Error> {
    let mut children = Vec::new();
    children
        .try_reserve(parent.children.len() + 1)
        .map_err(|_| Error::Limit {
            span,
            message: "async statement location allocation failed".into(),
        })?;
    children.extend_from_slice(&parent.children);
    children.push(child);
    Ok(StatementLocation {
        root: parent.root,
        children,
    })
}
