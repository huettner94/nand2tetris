use chumsky::prelude::*;
use text::{inline_whitespace, newline};

use super::{PopDest, PushSource, Statement};

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

fn int<'a>() -> impl Parser<'a, &'a str, u16, extra::Err<Rich<'a, char, Span>>> {
    text::int(10)
        .padded_by(inline_whitespace())
        .map(|s: &str| u16::from_str_radix(s, 10).unwrap())
        .boxed()
}

fn push<'a>(
    filename: &str,
) -> impl Parser<'a, &'a str, Statement, extra::Err<Rich<'a, char, Span>>> {
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
        .ignore_then(source)
        .then(int())
        .map(|(s, n)| Statement::Push(s, n))
        .boxed()
}

fn pop<'a>(
    filename: &str,
) -> impl Parser<'a, &'a str, Statement, extra::Err<Rich<'a, char, Span>>> {
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
        .ignore_then(dest)
        .then(int())
        .map(|(d, n)| Statement::Pop(d, n))
        .boxed()
}

fn branching<'a>() -> impl Parser<'a, &'a str, Statement, extra::Err<Rich<'a, char, Span>>> {
    let label = text::ident().map(|s: &str| s.to_string());

    choice((
        just("label")
            .padded_by(inline_whitespace())
            .ignore_then(label)
            .map(|label| Statement::Label(label)),
        just("goto")
            .padded_by(inline_whitespace())
            .ignore_then(label)
            .map(|label| Statement::Goto(label)),
        just("if-goto")
            .padded_by(inline_whitespace())
            .ignore_then(label)
            .map(|label| Statement::IfGoto(label)),
    ))
}

pub fn parser<'a>(
    filename: &str,
) -> impl Parser<'a, &'a str, Vec<Spanned<Statement>>, extra::Err<Rich<'a, char, Span>>> {
    let comment =
        inline_whitespace().ignore_then(just("//").then(any().and_is(newline().not()).repeated()));

    let opt_comment_and_newline = comment
        .repeated()
        .at_most(1)
        .ignore_then(newline())
        .ignore_then(inline_whitespace());

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
        branching(),
    ));

    line.map_with(|tok, e| (tok, e.span()))
        .padded_by(opt_comment_and_newline.repeated())
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}
