use std::borrow::Cow;

/// strip cuts the string to a specific width.
///
/// Width is expected to be in bytes.
pub fn cut_str(s: &str, width: usize) -> Cow<'_, str> {
    const REPLACEMENT: char = '\u{FFFD}';

    #[cfg(feature = "color")]
    {
        let stripped = ansi_str::AnsiStr::ansi_strip(s);
        let (length, count_unknowns, _) = string_split_at_length(&stripped, width);

        let mut buf = ansi_str::AnsiStr::ansi_cut(s, ..length);
        buf.extend(std::iter::repeat(REPLACEMENT).take(count_unknowns));

        Cow::Owned(buf)
    }
    #[cfg(not(feature = "color"))]
    {
        let (length, count_unknowns, _) = string_split_at_length(s, width);
        let buf = &s[..length];
        if count_unknowns == 0 {
            return Cow::Borrowed(buf);
        }

        let mut buf = buf.to_owned();
        buf.extend(std::iter::repeat(REPLACEMENT).take(count_unknowns));

        Cow::Owned(buf)
    }
}

pub fn string_split_at_length(s: &str, width: usize) -> (usize, usize, usize) {
    let mut length = 0;
    let mut i = 0;
    for c in s.chars() {
        if i == width {
            break;
        };

        let c_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);

        // We cut the chars which takes more then 1 symbol to display,
        // in order to archive the necessary width.
        if i + c_width > width {
            let count = width - i;
            return (length, count, c.len_utf8());
        }

        i += c_width;
        length += c.len_utf8();
    }

    (length, 0, 0)
}

/// Returns a string width.
#[cfg(not(feature = "color"))]
pub fn string_width(text: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(text)
}

/// Returns a string width.
#[cfg(feature = "color")]
pub fn string_width(text: &str) -> usize {
    let b = strip_ansi_escapes::strip(text.as_bytes()).unwrap();
    let s = std::str::from_utf8(&b).unwrap();
    unicode_width::UnicodeWidthStr::width(s)
}

/// Returns a max string width of a line.
#[cfg(not(feature = "color"))]
pub fn string_width_multiline(text: &str) -> usize {
    text.lines()
        .map(unicode_width::UnicodeWidthStr::width)
        .max()
        .unwrap_or(0)
}

/// Returns a max string width of a line.
#[cfg(feature = "color")]
pub fn string_width_multiline(text: &str) -> usize {
    let b = strip_ansi_escapes::strip(text.as_bytes()).unwrap();
    let s = std::str::from_utf8(&b).unwrap();

    s.lines()
        .map(unicode_width::UnicodeWidthStr::width)
        .max()
        .unwrap_or(0)
}

pub fn count_lines(s: &str) -> usize {
    if s.is_empty() {
        return 1;
    }

    bytecount::count(s.as_bytes(), b'\n') + 1
}

pub fn string_width_tab(text: &str, tab_width: usize) -> usize {
    let width = string_width(text);
    let count_tabs = count_tabs(text);

    width + count_tabs * tab_width
}

pub fn string_width_multiline_tab(text: &str, tab_width: usize) -> usize {
    text.lines()
        .map(|line| string_width_tab(line, tab_width))
        .max()
        .unwrap_or(0)
}

#[cfg(not(feature = "color"))]
pub fn string_trim(text: &str) -> Cow<'_, str> {
    text.trim().into()
}

#[cfg(feature = "color")]
pub fn string_trim(text: &str) -> Cow<'_, str> {
    ansi_str::AnsiStr::ansi_trim(text).into()
}

pub fn count_tabs(s: &str) -> usize {
    bytecount::count(s.as_bytes(), b'\t')
}

#[cfg(not(feature = "color"))]
pub fn get_lines(text: &str) -> impl Iterator<Item = Cow<'_, str>> {
    // we call split but not `lines()` in order to match colored implementation
    text.split('\n').map(Cow::Borrowed)
}

#[cfg(feature = "color")]
pub fn get_lines(text: &str) -> impl Iterator<Item = Cow<'_, str>> {
    ansi_str::AnsiStr::ansi_split(text, "\n")
}

pub fn replace_tab(text: &str, n: usize) -> String {
    // it's a general case which probably must be faster?
    if n == 4 {
        text.replace('\t', "    ")
    } else {
        let mut text = text.to_owned();
        replace_tab_range(&mut text, n);
        text
    }
}

fn replace_tab_range(cell: &mut String, n: usize) -> &str {
    let mut skip = 0;
    while let &Some(pos) = &cell[skip..].find('\t') {
        let pos = skip + pos;

        let is_escaped = pos > 0 && cell.get(pos - 1..pos) == Some("\\");
        if is_escaped {
            skip = pos + 1;
        } else if n == 0 {
            cell.remove(pos);
            skip = pos;
        } else {
            // I'am not sure which version is faster a loop of 'replace'
            // or allacation of a string for replacement;
            cell.replace_range(pos..=pos, &" ".repeat(n));
            skip = pos + 1;
        }

        if cell.is_empty() || skip >= cell.len() {
            break;
        }
    }
    cell
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_tab_test() {
        assert_eq!(replace_tab("123\t\tabc\t", 3), "123      abc   ");

        assert_eq!(replace_tab("\t", 0), "");
        assert_eq!(replace_tab("\t", 3), "   ");
        assert_eq!(replace_tab("123\tabc", 3), "123   abc");
        assert_eq!(replace_tab("123\tabc\tzxc", 0), "123abczxc");

        assert_eq!(replace_tab("\\t", 0), "\\t");
        assert_eq!(replace_tab("\\t", 4), "\\t");
        assert_eq!(replace_tab("123\\tabc", 0), "123\\tabc");
        assert_eq!(replace_tab("123\\tabc", 4), "123\\tabc");
    }

    #[test]
    fn string_width_emojie_test() {
        // ...emojis such as “joy”, which normally take up two columns when printed in a terminal
        // https://github.com/mgeisler/textwrap/pull/276
        assert_eq!(string_width("🎩"), 2);
        assert_eq!(string_width("Rust 💕"), 7);
        assert_eq!(string_width_multiline("Go 👍\nC 😎"), 5);
    }

    #[cfg(feature = "color")]
    #[test]
    fn colored_string_width_test() {
        use owo_colors::OwoColorize;
        assert_eq!(string_width(&"hello world".red().to_string()), 11);
        assert_eq!(
            string_width_multiline(&"hello\nworld".blue().to_string()),
            5
        );
        assert_eq!(string_width("\u{1b}[34m0\u{1b}[0m"), 1);
        assert_eq!(string_width(&"0".red().to_string()), 1);
    }

    #[test]
    fn strip_test() {
        assert_eq!(cut_str("123456", 0), "");
        assert_eq!(cut_str("123456", 3), "123");
        assert_eq!(cut_str("123456", 10), "123456");

        assert_eq!(cut_str("a week ago", 4), "a we");

        assert_eq!(cut_str("😳😳😳😳😳", 0), "");
        assert_eq!(cut_str("😳😳😳😳😳", 3), "😳�");
        assert_eq!(cut_str("😳😳😳😳😳", 4), "😳😳");
        assert_eq!(cut_str("😳😳😳😳😳", 20), "😳😳😳😳😳");

        assert_eq!(cut_str("🏳️🏳️", 0), "");
        assert_eq!(cut_str("🏳️🏳️", 1), "🏳");
        assert_eq!(cut_str("🏳️🏳️", 2), "🏳\u{fe0f}🏳");
        assert_eq!(string_width("🏳️🏳️"), string_width("🏳\u{fe0f}🏳"));

        assert_eq!(cut_str("🎓", 1), "�");
        assert_eq!(cut_str("🎓", 2), "🎓");

        assert_eq!(cut_str("🥿", 1), "�");
        assert_eq!(cut_str("🥿", 2), "🥿");

        assert_eq!(cut_str("🩰", 1), "�");
        assert_eq!(cut_str("🩰", 2), "🩰");

        assert_eq!(cut_str("👍🏿", 1), "�");
        assert_eq!(cut_str("👍🏿", 2), "👍");
        assert_eq!(cut_str("👍🏿", 3), "👍�");
        assert_eq!(cut_str("👍🏿", 4), "👍🏿");

        assert_eq!(cut_str("🇻🇬", 1), "🇻");
        assert_eq!(cut_str("🇻🇬", 2), "🇻🇬");
        assert_eq!(cut_str("🇻🇬", 3), "🇻🇬");
        assert_eq!(cut_str("🇻🇬", 4), "🇻🇬");
    }

    #[cfg(feature = "color")]
    #[test]
    fn strip_color_test() {
        use owo_colors::OwoColorize;

        let numbers = "123456".red().on_bright_black().to_string();

        assert_eq!(cut_str(&numbers, 0), "\u{1b}[31;100m\u{1b}[39m\u{1b}[49m");
        assert_eq!(
            cut_str(&numbers, 3),
            "\u{1b}[31;100m123\u{1b}[39m\u{1b}[49m"
        );
        assert_eq!(cut_str(&numbers, 10), "\u{1b}[31;100m123456\u{1b}[0m");

        let emojies = "😳😳😳😳😳".red().on_bright_black().to_string();

        assert_eq!(cut_str(&emojies, 0), "\u{1b}[31;100m\u{1b}[39m\u{1b}[49m");
        assert_eq!(
            cut_str(&emojies, 3),
            "\u{1b}[31;100m😳\u{1b}[39m\u{1b}[49m�"
        );
        assert_eq!(
            cut_str(&emojies, 4),
            "\u{1b}[31;100m😳😳\u{1b}[39m\u{1b}[49m"
        );
        assert_eq!(cut_str(&emojies, 20), "\u{1b}[31;100m😳😳😳😳😳\u{1b}[0m");

        let emojies = "🏳️🏳️".red().on_bright_black().to_string();

        assert_eq!(cut_str(&emojies, 0), "\u{1b}[31;100m\u{1b}[39m\u{1b}[49m");
        assert_eq!(cut_str(&emojies, 1), "\u{1b}[31;100m🏳\u{1b}[39m\u{1b}[49m");
        assert_eq!(
            cut_str(&emojies, 2),
            "\u{1b}[31;100m🏳\u{fe0f}🏳\u{1b}[39m\u{1b}[49m"
        );
        assert_eq!(
            string_width(&emojies),
            string_width("\u{1b}[31;100m🏳\u{fe0f}🏳\u{1b}[39m\u{1b}[49m")
        );
    }

    #[test]
    fn count_lines_test() {
        assert_eq!(
            count_lines("\u{1b}[37mnow is the time for all good men\n\u{1b}[0m"),
            2
        );
        assert_eq!(count_lines("now is the time for all good men\n"), 2);
    }
}
