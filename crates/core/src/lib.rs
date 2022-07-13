pub mod language;
mod matcher;
mod meta_var;
mod node;
mod pattern;
mod replacer;
mod rule;
mod ts_parser;

pub use meta_var::MetaVarMatcher;
pub use node::Node;
pub use pattern::Pattern;
pub use rule::{All, Either, Rule};
pub use rule::Matcher;

use crate::{replacer::Replacer, rule::PositiveMatcher};
use language::Language;
use node::Root;
use ts_parser::Edit;

pub struct AstGrep<L: Language> {
    inner: Root<L>,
}

impl<L: Language> AstGrep<L> {
    pub fn new<S: AsRef<str>>(src: S, lang: L) -> Self {
        Self {
            inner: Root::new(src.as_ref(), lang),
        }
    }

    pub fn root(&self) -> Node<L> {
        self.inner.root()
    }

    pub fn edit(&mut self, edit: Edit) -> &mut Self {
        self.inner.do_edit(edit);
        self
    }

    pub fn replace<M: PositiveMatcher<L>, R: Replacer<L>>(
        &mut self,
        pattern: M,
        replacer: R,
    ) -> bool {
        if let Some(edit) = self.root().replace(pattern, replacer) {
            self.edit(edit);
            true
        } else {
            false
        }
    }

    pub fn generate(self) -> String {
        self.inner.source
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use language::Tsx;
    #[test]
    fn test_replace() {
        let mut ast_grep = Tsx.new("var a = 1; let b = 2;");
        ast_grep.replace("var $A = $B", "let $A = $B");
        let source = ast_grep.generate();
        assert_eq!(source, "let a = 1; let b = 2;"); // note the semicolon
    }

    #[test]
    fn test_replace_by_rule() {
        let rule = Rule::either("let a = 123").or("let b = 456").build();
        let mut ast_grep = Tsx.new("let a = 123");
        let replaced = ast_grep.replace(rule, "console.log('it works!')");
        assert!(replaced);
        let source = ast_grep.generate();
        assert_eq!(source, "console.log('it works!')");
    }

    #[test]
    fn test_replace_trivia() {
        let mut ast_grep = Tsx.new("var a = 1 /*haha*/;");
        ast_grep.replace("var $A = $B", "let $A = $B");
        let source = ast_grep.generate();
        assert_eq!(source, "let a = 1;"); // semicolon

        let mut ast_grep = Tsx.new("var a = 1; /*haha*/");
        ast_grep.replace("var $A = $B", "let $A = $B");
        let source = ast_grep.generate();
        assert_eq!(source, "let a = 1; /*haha*/");
    }
}