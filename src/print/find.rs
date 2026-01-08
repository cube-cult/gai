use dialoguer::{Select, theme::ColorfulTheme};

use crate::schema::find::FindCommitSchema;

pub fn print(commit: FindCommitSchema) -> anyhow::Result<usize> {
    let options = ["Checkout", "Retry", "Exit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option:")
        .items(options)
        .default(0)
        .interact()?;

    Ok(selection)
}
