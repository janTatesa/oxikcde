use crate::SwitchToComic;
use clap::{
    builder::{styling::AnsiColor::*, Styles},
    command, value_parser, Arg, ArgAction, ArgMatches,
};
use color_eyre::{eyre::ContextCompat, Result};
use std::{ffi::OsString, path::PathBuf};
use tap::Tap;

const STYLE: Styles = Styles::styled()
    .header(Green.on_default().bold())
    .usage(Green.on_default().bold())
    .literal(Blue.on_default().bold())
    .placeholder(Cyan.on_default());

pub fn cli() -> Result<ArgMatches> {
    Ok(command!()
        .args([
            Arg::new("number")
                .value_parser(value_parser!(u16))
                .required(false)
                .conflicts_with("initial_comic"),
            Arg::new("initial_comic")
                .value_parser(value_parser!(SwitchToComic))
                .short('i')
                .help("The default value for this argument is the initial_comic config option"),
            Arg::new("config_path")
                .value_parser(value_parser!(PathBuf))
                .short('c')
                .default_value(default_config_path()?),
            Arg::new("print_default_config")
                .action(ArgAction::SetTrue)
                .short('p')
                .help("Print default config")
                .exclusive(true),
        ])
        .styles(STYLE)
        .get_matches())
}

fn default_config_path() -> Result<OsString> {
    Ok(dirs::config_dir()
        .wrap_err("Unsupported platform")?
        .tap_mut(|p| p.extend(["oxikcde", "oxikcde.toml"]))
        .into())
}
