use clap::{
    builder::{styling::AnsiColor::*, Styles},
    command, value_parser, Arg, ArgMatches,
};
const STYLE: Styles = Styles::styled()
    .header(Green.on_default().bold())
    .usage(Green.on_default().bold())
    .literal(Blue.on_default().bold())
    .placeholder(Cyan.on_default());

pub fn cli() -> ArgMatches {
    command!()
        .args([
            Arg::new("number")
                .value_parser(value_parser!(u64))
                .required(false),
            Arg::new("initial_comic")
                .value_parser(value_parser!(crate::app::SwitchToComic))
                .short('i'),
        ])
        .styles(STYLE)
        .get_matches()
}
