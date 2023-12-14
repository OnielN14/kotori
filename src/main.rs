use anyhow::Ok;
use clap::{arg, command};
use kotori::translate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = command!()
    .arg(arg!(-i --input <TEXT>).required(true))
    .arg(arg!(--from <LANG> "Check https://cloud.google.com/translate/docs/languages for Language code").required(true))
    .arg(arg!(--to <LANG> "Check https://cloud.google.com/translate/docs/languages for Language code").required(true))
    .arg(arg!(--useragent <USER_AGENT> "Customize user agent").required(false))
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

    let result = translate(
        input,
        &from,
        &to,
        cli.get_one::<String>("useragent").cloned(),
    )
    .await?;

    println!("{}", result);
    Ok(())
}
