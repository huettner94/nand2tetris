use chumsky::prelude::*;
use text::{inline_whitespace, newline};

use super::{PushSource, Statement};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Statement>> {
    let comment = just("//").then(any().and_is(newline().not()).repeated());

    let opt_comment_and_newline = comment.repeated().at_most(1).ignore_then(newline());

    let int = text::int(10)
        .padded_by(inline_whitespace())
        .map(|s: &str| u16::from_str_radix(s, 10).unwrap());

    let pc = just("push")
        .padded_by(inline_whitespace())
        .ignore_then(just("constant").padded_by(inline_whitespace()))
        .ignore_then(int)
        .map(|n| Statement::Push(PushSource::Constant, n));

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
        pc,
    ));

    line.separated_by(opt_comment_and_newline.repeated())
        .allow_trailing()
        .allow_leading()
        .collect()
}
