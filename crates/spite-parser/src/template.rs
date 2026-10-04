//! Template components, tagged call sites, and restricted escapes (13.2.8, 13.3.11).

use crate::{Diagnostic, Expr, ExprKind, Kind, Parser, Span, TemplateElement, early};
use std::rc::Rc;

impl Parser {
    pub(super) fn template_literal(
        &mut self,
        element: TemplateElement,
        mut tail: bool,
        span: Span,
        tag: Option<Expr>,
    ) -> Result<Expr, Diagnostic> {
        let mut elements = vec![element];
        let mut substitutions = Vec::new();
        let mut end = span.end;
        while !tail {
            substitutions.push(self.expression_with_in(1, true)?);
            let token = self.bump();
            let Kind::Template {
                element,
                tail: is_tail,
                continuation: true,
            } = token.kind
            else {
                return Err(early(token.span, "expected template substitution tail"));
            };
            elements.push(element);
            tail = is_tail;
            end = token.span.end;
        }
        // 13.2.8.1: the number of TemplateStrings must be less than 2^32.
        if u32::try_from(elements.len()).is_err() {
            return Err(early(span, "too many template components"));
        }
        let start = tag.as_ref().map_or(span.start, |tag| tag.span.start);
        let kind = if let Some(tag) = tag {
            ExprKind::TaggedTemplate {
                tag: Box::new(tag),
                elements: Rc::from(elements),
                substitutions,
            }
        } else {
            for element in &elements {
                if element.cooked.is_none() {
                    return Err(early(element.span, "invalid escape in untagged template"));
                }
            }
            ExprKind::Template {
                elements,
                substitutions,
            }
        };
        self.make_expr(kind, Span::new(start, end))
    }
}
