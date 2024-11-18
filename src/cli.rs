use clap::{
    builder::{styling, Styles},
    value_parser, Arg, ArgMatches, Command,
};
const STYLE: Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

pub fn cli() -> ArgMatches {
    Command::new("oxikcde")
        .arg(
            Arg::new("number")
                .value_parser(value_parser!(u64))
                .required(false),
        )
        .arg(
            Arg::new("initial_comic")
                .value_parser(value_parser!(crate::app::SwitchToComic))
                .short('i'),
        )
        .styles(STYLE)
        .get_matches()
}
