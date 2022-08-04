use clap::Parser;
use strum_macros::Display;
use strum_macros::EnumString;

#[derive(Parser)]
pub struct Cli {
    #[clap(long, default_value_t = Format::debug)]
    pub format: Format,
}

#[derive(EnumString, Display)]
#[allow(non_camel_case_types)]
pub enum Format {
    bson,
    debug,
}
