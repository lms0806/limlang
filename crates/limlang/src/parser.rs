mod event;
mod expr;
mod marker;
mod sink;
mod source;

use crate::lexer::{Lexeme, Lexer, SyntaxKind};
use crate::parser::marker::Marker;
use crate::parser::source::Source;
use crate::syntax::SyntaxNode;
use event::Event;
use expr::expr;
use rowan::GreenNode;
use sink::Sink;

struct Parser<'l, 'input> {
    source: Source<'l, 'input>,
    events: Vec<Event>,
}

impl<'l, 'input> Parser<'l, 'input> {
    fn new(lexemes: &'l [Lexeme<'input>]) -> Self {
        Self {
            source: Source::new(lexemes),
            events: Vec::new(),
        }
    }

    fn parse(mut self) -> Vec<Event> {
        let m = self.start();
        expr(&mut self);
        m.complete(&mut self, SyntaxKind::Root);

        self.events
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.events.push(Event::StartNode {
            kind,
            forward_parent: None,
        });
    }

    fn start_node_at(&mut self, checkpoint: usize, kind: SyntaxKind) {
        self.events.push(Event::StartNodeAt { kind, checkpoint });
    }

    fn finish_node(&mut self) {
        self.events.push(Event::FinishNode);
    }

    fn bump(&mut self) {
        let Lexeme { kind, text } = self.source.next_lexeme().unwrap();

        self.events.push(Event::AddToken {
            kind: *kind,
            text: (*text).into(),
        });
    }

    fn checkpoint(&self) -> usize {
        self.events.len()
    }

    fn peek(&mut self) -> Option<SyntaxKind> {
        self.source.peek_kind()
    }

    fn start(&mut self) -> Marker {
        let pos = self.events.len();
        self.events.push(Event::Placeholder);

        Marker::new(pos)
    }
}

pub fn parse(input: &str) -> Parse {
    let lexemes: Vec<_> = Lexer::new(input).collect();
    let parser = Parser::new(&lexemes);
    let events = parser.parse();
    let sink = Sink::new(&lexemes, events);

    Parse {
        green_node: sink.finish(),
    }
}

pub struct Parse {
    green_node: GreenNode,
}

impl Parse {
    pub fn debug_tree(&self) -> String {
        let syntax_node = SyntaxNode::new_root(self.green_node.clone());
        let formatted = format!("{:#?}", syntax_node);

        formatted[0..formatted.len() - 1].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::super::check;
    use expect_test::expect;

    #[test]
    fn parse_nothing() {
        check("", expect![[r#"Root@0..0"#]]);
    }

    #[test]
    fn parse_number() {
        check(
            "123",
            expect![[r#"
Root@0..3
  Literal@0..3
    Number@0..3 "123""#]],
        );
    }

    #[test]
    fn parse_binding_usage() {
        check(
            "counter",
            expect![[r#"
Root@0..7
  VariableRef@0..7
    Ident@0..7 "counter""#]],
        );
    }

    #[test]
    fn parse_whitespace() {
        check(
            "   ",
            expect![[r#"
Root@0..3
  Whitespace@0..3 "   ""#]],
        );
    }

    #[test]
    fn parse_comment() {
        check(
            "# hello!",
            expect![[r##"
Root@0..8
  Comment@0..8 "# hello!""##]],
        );
    }
}
