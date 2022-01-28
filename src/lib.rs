use logos::Logos;

mod lexer {
    use logos::Logos;

    #[derive(Logos, Clone, Eq, PartialEq, Hash)]
    pub enum Token<'a> {
        #[token("λ")]
        Lambda,

        #[token(".")]
        Dot,

        #[token(":=")]
        Binding,

        #[token("(")]
        ParenO,

        #[token(")")]
        ParenC,

        #[regex("[a-zA-Z]+")]
        Ident(&'a str),

        #[error]
        #[regex(r"[ \t\r\n]+", logos::skip)]
        Error,
    }
}

mod parser {
    use crate::lexer::Token;
    use chumsky::prelude::*;

    enum Expr {
        Name(String),
        Application {
            function: Box<Expr>,
            argument: Box<Expr>,
        },
        Abstraction {
            args: Vec<String>,
            body: Expr,
        },
    }

    fn expr_parser<'a>() -> impl Parser<Token<'a>, Spanned<Expr>, Error = Simple<Token<'a>>> + Clone
    {
        recursive(|expr| {
            let variable = filter_map(|span, token| match token {
                Token::Ident(name) => Ok(Expr::Name(name.to_string())),
                _ => Err(Simple::expected_input_found(span, [], Some(token))),
            })
            .labelled("variable");

            let abstraction = just(Token::Lambda)
                .ignore_then(variable)
                .then(Token::Dot)
                .then(expr.clone());

            let application = just(expr).then(expr.clone());

            let atom = variable
                .or(expr.delimited_by(Token::ParenO, Token::ParenC))
                .or(abstraction)
                .or(application);

            let binding = just(variable)
                .then_ignore(Token::Binding)
                .then(expr.clone());

            let statement = just(expr.clone()).or(binding);

            todo!()
        })
    }
}

pub fn run(input: &str) {
    let mut lex = lexer::Token::lexer(input);
}
