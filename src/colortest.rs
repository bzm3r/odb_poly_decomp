use nu_ansi_term::AnsiGenericString;
use nu_ansi_term::{Color, Style};
use std::fmt;
use std::{
    borrow::Cow,
    fmt::{Arguments, Debug, Display},
};
use tracing::info;

// [`AnsiGenericString`](https://docs.rs/nu-ansi-term/latest/nu_ansi_term/struct.AnsiGenericString.html)
pub struct AnsiFormatArgs<'a> {
    fmt_args: Arguments<'a>,
    style: Style,
}

impl<'a> AnsiFormatArgs<'a> {
    pub fn new(fmt_args: Arguments<'a>, style: Style) -> Self {
        Self { fmt_args, style }
    }
}

impl<'a> Display for AnsiFormatArgs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.style.prefix())?;
        write!(f, "{}", self.fmt_args)?;
        write!(f, "{}", self.style.suffix())
    }
}

// Uses [`nu_ansi_term`](https://docs.rs/nu-ansi-term/latest/nu_ansi_term/)
//
// ansi feature is being used
fn main() {
    tracing_subscriber::fmt()
        .pretty()
        // enable everything
        .with_max_level(tracing::Level::TRACE)
        // sets this to be the default, global collector for this application.
        .init();
    let red = Style::new().reset_before_style().fg(Color::Red);
    info!("not using AnsiFormatArgs: {}", red.paint("hello world!"));
    info!(
        "using AnsiFormatArgs: {}",
        AnsiFormatArgs::new(format_args!("{}", "hello world!"), red)
    );
}
