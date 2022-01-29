use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{Parser, Stream};
use logos::Logos;

mod lexer {
    use logos::Logos;
    use std::fmt::Formatter;

    #[derive(Logos, Debug, Clone, Eq, PartialEq, Hash)]
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

    impl std::fmt::Display for Token<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Token::Lambda => write!(f, "λ"),
                Token::Dot => write!(f, "."),
                Token::Binding => write!(f, ":="),
                Token::ParenO => write!(f, "("),
                Token::ParenC => write!(f, ")"),
                Token::Ident(ident) => write!(f, "{}", ident),
                Token::Error => write!(f, "[error]"),
            }
        }
    }
}

mod parser {
    use crate::lexer::Token;
    use chumsky::prelude::*;

    #[derive(Debug)]
    pub enum Expr {
        Name(String),
        Application {
            function: Box<Expr>,
            argument: Box<Expr>,
        },
        Abstraction {
            args: Vec<char>,
            body: Box<Expr>,
        },
    }

    pub fn expr_parser<'a>() -> impl Parser<Token<'a>, Expr, Error = Simple<Token<'a>>> + Clone {
        recursive(|expr| {
            let ident = filter_map(|span, token| match token {
                Token::Ident(ident) => Ok(ident.to_string()),
                _ => Err(Simple::expected_input_found(span, [], Some(token))),
            })
            .labelled("ident");

            let abstraction = just(Token::Lambda)
                .ignore_then(ident)
                .then_ignore(just(Token::Dot))
                .then(expr)
                .map(|(args, body)| Expr::Abstraction {
                    args: args.chars().collect(),
                    body: Box::new(body),
                })
                .labelled("abstraction");

            abstraction
        })
    }
}

pub fn run(input: &str) {
    let mut lexer = lexer::Token::lexer(input);
    let length = lexer.source().len();

    match parser::expr_parser().parse(Stream::from_iter(length..length + 1, lexer.spanned())) {
        Ok(ast) => println!("parsed: {ast:#?}"),
        Err(errs) => errs
            .into_iter()
            .map(|e| e.map(|c| c.to_string()))
            .for_each(|e| {
                let report = Report::build(ReportKind::Error, (), e.span().start);

                let report = match e.reason() {
                    chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                        .with_message(format!(
                            "Unclosed delimiter {}",
                            delimiter.fg(Color::Yellow)
                        ))
                        .with_label(
                            Label::new(span.clone())
                                .with_message(format!(
                                    "Unclosed delimiter {}",
                                    delimiter.fg(Color::Yellow)
                                ))
                                .with_color(Color::Yellow),
                        )
                        .with_label(
                            Label::new(e.span())
                                .with_message(format!(
                                    "Must be closed before this {}",
                                    e.found()
                                        .unwrap_or(&"end of file".to_string())
                                        .fg(Color::Red)
                                ))
                                .with_color(Color::Red),
                        ),
                    chumsky::error::SimpleReason::Unexpected => report
                        .with_message(format!(
                            "{}, expected {}",
                            if e.found().is_some() {
                                "Unexpected token in input"
                            } else {
                                "Unexpected end of input"
                            },
                            if e.expected().len() == 0 {
                                "something else".to_string()
                            } else {
                                e.expected()
                                    .map(|expected| match expected {
                                        Some(expected) => expected.to_string(),
                                        None => "end of input".to_string(),
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            }
                        ))
                        .with_label(
                            Label::new(e.span())
                                .with_message(format!(
                                    "Unexpected token {}",
                                    e.found()
                                        .unwrap_or(&"end of file".to_string())
                                        .fg(Color::Red)
                                ))
                                .with_color(Color::Red),
                        ),
                    chumsky::error::SimpleReason::Custom(msg) => {
                        report.with_message(msg).with_label(
                            Label::new(e.span())
                                .with_message(format!("{}", msg.fg(Color::Red)))
                                .with_color(Color::Red),
                        )
                    }
                };

                report.finish().print(Source::from(input)).unwrap();
            }),
    }
}
