use std::{
    borrow::Cow,
    fmt::{self, Arguments, Debug, Display},
};

use nu_ansi_term::{AnsiGenericString, AnsiString};

pub struct PaintFormatArgs<'a>(Arguments<'a>);

impl<'a> PaintFormatArgs<'a> {
    pub fn new(args: Arguments<'a>) -> Self {
        Self(args)
    }
}

impl<'a> Debug for PaintFormatArgs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> ToOwned for PaintFormatArgs<'a> {
    type Owned = PaintFormatArgs<'a>;
    fn to_owned(&self) -> Self::Owned {
        PaintFormatArgs(self.0)
    }
}

impl<'a> From<PaintFormatArgs<'a>> for Cow<'a, PaintFormatArgs<'a>>
where
    PaintFormatArgs<'a>: ToOwned,
    <PaintFormatArgs<'a> as ToOwned>::Owned: Debug,
{
    fn from(value: PaintFormatArgs<'a>) -> Self {
        Cow::Owned(value.to_owned())
    }
}

impl<'a> Display for PaintFormatArgs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct AnsiFormatArgs<'a>(pub AnsiGenericString<'a, PaintFormatArgs<'a>>);

impl<'a> Display for AnsiFormatArgs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

pub struct TracingFormatter;

#[macro_export]
macro_rules! loop_span {
    (sty:$style:ident, id:$loop_id:literal) => {
        tracing::info_span!($loop_id, "{}", $style.paint("=============="))
    };
}

#[macro_export]
macro_rules! emit_info {
    (sty:$style:ident, msg:$msg:literal) => {
        tracing::info!("{}", $style.paint($msg));
    };
    (sty:$style:ident, fmt:$fmt:literal | $($rest:tt)+) => {
        tracing::info!("{}", $style.paint($fmt, $($rest)*));
    };
    (fmt:$fmt:literal | $($rest:tt)+) => {
        tracing::info!($fmt, $($rest)*);
    };
}

#[macro_export]
macro_rules! emit_info_span {
    (sty:$style:ident, msg:$msg:literal) => {
        tracing::info_span!("{}", $style.paint($msg));
    };
    (sty:$style:ident, fmt:$fmt:literal | $($rest:tt)+) => {
        tracing::info_span!("{}", $style.paint($fmt, $($rest)*));
    };
    (fmt:$fmt:literal | $($rest:tt)+) => {
        tracing::info_span!($fmt, $($rest)*);
    };
}

use nu_ansi_term::{Color, Style};

#[derive(Clone, Copy, Debug)]
pub struct MiniStyle {
    pub prefix_with_reset: bool,
    pub foreground: Option<Color>,
    pub is_bold: bool,
    pub is_underline: bool,
}

impl MiniStyle {
    pub fn override_fg(&self, color: Option<Color>) -> Self {
        let mut result = *self;
        color.and_then(|color| result.foreground.replace(color));
        result
    }
}

impl Default for MiniStyle {
    fn default() -> Self {
        Self {
            prefix_with_reset: true,
            foreground: None,
            is_bold: false,
            is_underline: false,
        }
    }
}

pub const COLOR_GREEN: Color = Color::Fixed(40);
pub const COLOR_BLUE: Color = Color::Fixed(27);
pub const COLOR_ORANGE: Color = Color::Fixed(208);

pub const STYLE_CURSOR: MiniStyle = MiniStyle {
    prefix_with_reset: true,
    foreground: None,
    is_bold: true,
    is_underline: true,
};

pub const LEFT_EDGE: MiniStyle = MiniStyle {
    prefix_with_reset: true,
    foreground: Some(COLOR_GREEN),
    is_bold: false,
    is_underline: false,
};

pub const RIGHT_EDGE: MiniStyle = MiniStyle {
    prefix_with_reset: true,
    foreground: Some(COLOR_BLUE),
    is_bold: false,
    is_underline: false,
};

impl From<MiniStyle> for Style {
    #[inline]
    fn from(mini: MiniStyle) -> Self {
        let MiniStyle {
            prefix_with_reset,
            foreground,
            is_bold,
            is_underline,
        } = mini;
        Self {
            prefix_with_reset,
            foreground,
            is_bold,
            is_underline,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct DebugItem<T: Debug + Copy> {
    pub style: MiniStyle,
    pub data: T,
}

impl<'a, T: Debug + Copy> From<&'a DebugItem<T>> for AnsiString<'a> {
    fn from(debug_string: &'a DebugItem<T>) -> Self {
        Style::from(debug_string.style)
            .paint(format!("{:?}", debug_string.data))
    }
}

impl<T: Debug + Copy> Debug for DebugItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", AnsiString::from(self))
    }
}

pub fn debug_with(
    f: impl Fn(&mut fmt::Formatter) -> fmt::Result,
) -> impl fmt::Debug {
    struct DebugWith<F>(F);

    impl<F> fmt::Debug for DebugWith<F>
    where
        F: Fn(&mut fmt::Formatter) -> fmt::Result,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0(f)
        }
    }

    DebugWith(f)
}
