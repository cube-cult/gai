use console::style;
use dialoguer::{Confirm, theme::ColorfulTheme};
use serde_json::Value;

use crate::{
    args::{FindArgs, GlobalArgs},
    git::log::get_logs,
    print::{InputHistory, loading, print_query_prompt},
    providers::{extract_from_provider, provider::ProviderKind},
    requests::find::create_find_request,
    responses::find::parse_from_schema,
    schema::{SchemaSettings, find::create_find_schema},
    state::State,
};

pub fn run(
    args: &FindArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let mut state = State::new(None)?;

    let count = args
        .number
        .unwrap_or_default();

    // todo add global args overrider
    if let Some(provider) = global.provider {
        state
            .settings
            .provider = provider;
    }

    let logs = get_logs(&state.git.repo, count, args.reverse)?;

    let schema_settings = if matches!(
        state
            .settings
            .provider,
        ProviderKind::OpenAI
    ) {
        SchemaSettings::default()
            .additional_properties(false)
            .allow_min_max_ints(true)
    } else {
        SchemaSettings::default().allow_min_max_ints(true)
    };

    let count = if logs.git_logs.len() > count {
        logs.git_logs.len() as u32
    } else {
        count as u32
    };

    let schema = create_find_schema(schema_settings, count)?;

    let mut history = InputHistory::default();

    loop {
        let q = if let Some(q) =
            print_query_prompt("What is your query?", &mut history)?
        {
            q
        } else {
            println!("Exiting...");
            break;
        };

        let text = format!(
            "Querying through your commits for \"{}\"",
            style(q.to_owned())
                .cyan()
                .bold()
        );

        let loading = loading::Loading::new(&text, global.compact)?;

        let req =
            create_find_request(&state.settings, &logs.git_logs, &q);

        loading.start();

        let response: Value = match extract_from_provider(
            &state
                .settings
                .provider,
            req.to_owned(),
            schema.to_owned(),
        ) {
            Ok(r) => r,
            Err(e) => {
                loading.stop();
                println!(
                    "Done but Gai received an error from the provider: {:#}",
                    e
                );

                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Retry?")
                    .interact()?
                {
                    continue;
                } else {
                    break;
                }
            }
        };

        let result = parse_from_schema(response)?;

        loading.stop();
    }

    Ok(())
}
