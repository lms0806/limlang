mod lexer;
pub mod parser;
mod syntax;

use crate::parser::Parser;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}

#[cfg(test)]
fn check(input: &str, expected_tree: expect_test::Expect) {
  let parse = Parser::new(input).parse();
  expected_tree.assert_eq(&parse.debug_tree());
}