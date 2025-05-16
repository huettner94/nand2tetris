use chumsky::prelude::*;
use text::{inline_whitespace, keyword, newline};

use super::{Ast, Function, PopDest, PushSource, Statement};

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

fn nl<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char, Span>>> {
    let comment =
        inline_whitespace().ignore_then(just("//").then(any().and_is(newline().not()).repeated()));

    let opt_comment_and_newline = comment
        .repeated()
        .at_most(1)
        .ignore_then(newline())
        .ignore_then(inline_whitespace());

    opt_comment_and_newline.repeated()
}

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
        keyword("constant")
            .padded_by(inline_whitespace())
            .to(PushSource::Constant),
        keyword("local")
            .padded_by(inline_whitespace())
            .to(PushSource::Local),
        keyword("argument")
            .padded_by(inline_whitespace())
            .to(PushSource::Argument),
        keyword("this")
            .padded_by(inline_whitespace())
            .to(PushSource::This),
        keyword("that")
            .padded_by(inline_whitespace())
            .to(PushSource::That),
        keyword("temp")
            .padded_by(inline_whitespace())
            .to(PushSource::Temp),
        keyword("pointer")
            .padded_by(inline_whitespace())
            .to(PushSource::Pointer),
        keyword("static")
            .padded_by(inline_whitespace())
            .to(PushSource::Static(filename.to_string())),
    ));

    keyword("push")
        .ignore_then(source)
        .then(int())
        .map(|(s, n)| Statement::Push(s, n))
        .boxed()
}

fn pop<'a>(
    filename: &str,
) -> impl Parser<'a, &'a str, Statement, extra::Err<Rich<'a, char, Span>>> {
    let dest = choice((
        keyword("local")
            .padded_by(inline_whitespace())
            .to(PopDest::Local),
        keyword("argument")
            .padded_by(inline_whitespace())
            .to(PopDest::Argument),
        keyword("this")
            .padded_by(inline_whitespace())
            .to(PopDest::This),
        keyword("that")
            .padded_by(inline_whitespace())
            .to(PopDest::That),
        keyword("temp")
            .padded_by(inline_whitespace())
            .to(PopDest::Temp),
        keyword("pointer")
            .padded_by(inline_whitespace())
            .to(PopDest::Pointer),
        keyword("static")
            .padded_by(inline_whitespace())
            .to(PopDest::Static(filename.to_string())),
    ));

    keyword("pop")
        .ignore_then(dest)
        .then(int())
        .map(|(d, n)| Statement::Pop(d, n))
        .boxed()
}

fn branching<'a>() -> impl Parser<'a, &'a str, Statement, extra::Err<Rich<'a, char, Span>>> {
    let label = text::ident().map(|s: &str| s.to_string());

    choice((
        keyword("label")
            .padded_by(inline_whitespace())
            .ignore_then(label)
            .map(|label| Statement::Label(label)),
        keyword("goto")
            .padded_by(inline_whitespace())
            .ignore_then(label)
            .map(|label| Statement::Goto(label)),
        just("if-goto")
            .padded_by(inline_whitespace())
            .ignore_then(label)
            .map(|label| Statement::IfGoto(label)),
    ))
}

pub fn statements<'a>(
    filename: &str,
) -> impl Parser<'a, &'a str, Vec<Statement>, extra::Err<Rich<'a, char, Span>>> {
    let line = choice((
        keyword("not").to(Statement::Not),
        keyword("and").to(Statement::And),
        keyword("or").to(Statement::Or),
        keyword("neg").to(Statement::Neg),
        keyword("add").to(Statement::Add),
        keyword("sub").to(Statement::Sub),
        keyword("eq").to(Statement::Eq),
        keyword("lt").to(Statement::Lt),
        keyword("gt").to(Statement::Gt),
        keyword("return").to(Statement::Return),
        push(filename),
        pop(filename),
        branching(),
    ));

    line.padded_by(nl()).repeated().collect()
}

pub fn functions<'a>(
    filename: &str,
) -> impl Parser<'a, &'a str, Vec<Function>, extra::Err<Rich<'a, char, Span>>> {
    let name = text::ident()
        .map(|s: &str| s.to_string())
        .then_ignore(just('.'))
        .then(text::ident().map(|s: &str| s.to_string()));

    let function = keyword("function")
        .padded_by(inline_whitespace())
        .ignore_then(name)
        .validate(|(funcfilename, funcname), span, emitter| {
            if !filename.starts_with(&funcfilename) {
                emitter.emit(Rich::custom(
                    span.span(),
                    format!(
                        "Function name first part must match the filename. Found '{}'.",
                        funcfilename
                    ),
                ));
            }
            format!("{}.{}", funcfilename, funcname)
        })
        .padded_by(inline_whitespace())
        .then(int())
        .padded_by(nl())
        .then(statements(filename))
        .map(|((name, locals), statements)| Function {
            name,
            locals,
            statements,
        });

    function.padded_by(nl()).repeated().collect()
}
