use dialoguer::{Input, theme::ColorfulTheme};

/// Input prompt
pub fn query(prompt: &str) -> anyhow::Result<String> {
    let s = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()?;

    Ok(s)
}
