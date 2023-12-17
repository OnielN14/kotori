use std::fmt::Display;

use anyhow::Ok;
use clap::builder::PossibleValue;
use clap::{arg, command, ValueEnum};
use kotori::google_translate::translate as google_translate;
use kotori::google_translate_m::translate as google_translate_m;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli().await?;

    Ok(())
}

#[derive(Clone, Debug)]
enum Machine {
    GoogleTranslate,
    GoogleTranslateM,
}

impl ValueEnum for Machine {
    fn value_variants<'a>() -> &'a [Self] {
        &[Machine::GoogleTranslate, Machine::GoogleTranslateM]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Machine::GoogleTranslate => {
                PossibleValue::new("google-translate").help("Using http://translatet.google.com")
            }
            Machine::GoogleTranslateM => PossibleValue::new("google-translate-m")
                .help("Using http://translatet.google.com/m (faster)"),
        })
    }
}

impl Display for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("No values are skipped")
            .get_name()
            .fmt(f)
    }
}

async fn cli() -> anyhow::Result<()> {
    let cli = command!()
        .arg(arg!(-i --input <TEXT>).required(true))
        .arg(arg!(--from <LANG> "Check https://cloud.google.com/translate/docs/languages for Language code").required(true))
        .arg(arg!(--to <LANG> "Check https://cloud.google.com/translate/docs/languages for Language code").required(true))
        .arg(arg!(--useragent <USER_AGENT> "Customize user agent").required(false))
        .arg(arg!(--machine <MACHINE_NAME>).help("Default using \"google-translate-m\"").value_parser(clap::value_parser!(Machine)))
    .get_matches();

    let (input, from, to): (String, String, String) = if let (Some(input), Some(from), Some(to)) = (
        cli.get_one::<String>("input"),
        cli.get_one::<String>("from"),
        cli.get_one::<String>("to"),
    ) {
        (input.clone(), from.clone(), to.clone())
    } else {
        eprintln!("Command Invalid");
        return Ok(());
    };

    let machine = cli
        .get_one::<Machine>("machine")
        .unwrap_or(&Machine::GoogleTranslateM);

    let result = match machine {
        Machine::GoogleTranslate => google_translate(input, &from, &to, None, None).await?,
        Machine::GoogleTranslateM => google_translate_m(input, &from, &to, None, None).await?,
    };

    println!("{}", result);
    Ok(())
}
