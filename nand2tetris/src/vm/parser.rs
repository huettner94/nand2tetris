use chumsky::prelude::*;
use text::{inline_whitespace, newline};

use super::{PopDest, PushSource, Statement};

fn int<'a>() -> impl Parser<'a, &'a str, u16> {
    text::int(10)
        .padded_by(inline_whitespace())
        .map(|s: &str| u16::from_str_radix(s, 10).unwrap())
        .boxed()
}

fn push<'a>(filename: &str) -> impl Parser<'a, &'a str, Statement> {
    let source = choice((
        just("constant")
            .padded_by(inline_whitespace())
            .to(PushSource::Constant),
        just("local")
            .padded_by(inline_whitespace())
            .to(PushSource::Local),
        just("argument")
            .padded_by(inline_whitespace())
            .to(PushSource::Argument),
        just("this")
            .padded_by(inline_whitespace())
            .to(PushSource::This),
        just("that")
            .padded_by(inline_whitespace())
            .to(PushSource::That),
        just("temp")
            .padded_by(inline_whitespace())
            .to(PushSource::Temp),
        just("pointer")
            .padded_by(inline_whitespace())
            .to(PushSource::Pointer),
        just("static")
            .padded_by(inline_whitespace())
            .to(PushSource::Static(filename.to_string())),
    ));

    just("push")
        .padded_by(inline_whitespace())
        .ignore_then(source)
        .then(int())
        .map(|(s, n)| Statement::Push(s, n))
        .boxed()
}

fn pop<'a>(filename: &str) -> impl Parser<'a, &'a str, Statement> {
    let dest = choice((
        just("local")
            .padded_by(inline_whitespace())
            .to(PopDest::Local),
        just("argument")
            .padded_by(inline_whitespace())
            .to(PopDest::Argument),
        just("this")
            .padded_by(inline_whitespace())
            .to(PopDest::This),
        just("that")
            .padded_by(inline_whitespace())
            .to(PopDest::That),
        just("temp")
            .padded_by(inline_whitespace())
            .to(PopDest::Temp),
        just("pointer")
            .padded_by(inline_whitespace())
            .to(PopDest::Pointer),
        just("static")
            .padded_by(inline_whitespace())
            .to(PopDest::Static(filename.to_string())),
    ));

    just("pop")
        .padded_by(inline_whitespace())
        .ignore_then(dest)
        .then(int())
        .map(|(d, n)| Statement::Pop(d, n))
        .boxed()
}

pub fn parser<'a>(filename: &str) -> impl Parser<'a, &'a str, Vec<Statement>> {
    let comment = just("//").then(any().and_is(newline().not()).repeated());

    let opt_comment_and_newline = comment.repeated().at_most(1).ignore_then(newline());

    let line = choice((
        just("not").to(Statement::Not),
        just("and").to(Statement::And),
        just("or").to(Statement::Or),
        just("neg").to(Statement::Neg),
        just("add").to(Statement::Add),
        just("sub").to(Statement::Sub),
        just("eq").to(Statement::Eq),
        just("lt").to(Statement::Lt),
        just("gt").to(Statement::Gt),
        push(filename),
        pop(filename),
    ));

    line.separated_by(opt_comment_and_newline.repeated())
        .allow_trailing()
        .allow_leading()
        .collect()
}
