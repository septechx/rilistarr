mod brawl_api;
mod commands;
mod data;
mod leaderboard;
mod permissions;

use poise::serenity_prelude as serenity;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
enum BotError {
    #[error("Failed to load .env: {0}")]
    DotEnv(#[from] dotenvy::Error),
    #[error("Missing environment variable: {0}")]
    EnvVar(#[from] env::VarError),
    #[error("Serenity error: {0}")]
    Serenity(String),
}

impl From<serenity::Error> for BotError {
    fn from(value: serenity::Error) -> Self {
        Self::Serenity(value.to_string())
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

async fn on_error(error: poise::FrameworkError<'_, (), Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            eprintln!("Error during framework setup: {}", error);
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            eprintln!("Error in command `{}`: {:?}", ctx.command().name, error);
            let _ = ctx.say(format!("❌ An error occurred: {}", error)).await;
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                eprintln!("Error while handling error: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), BotError> {
    if dotenvy::dotenv().is_err() {
        eprintln!("\x1b[33mwarning: Failed to load .env file\x1b[0m");
    }

    let token = env::var("DISCORD_TOKEN")?;
    let brawl_token = env::var("BRAWL_TOKEN")?;
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILDS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_commands(),
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                println!("Bot is starting up...");

                // Register slash commands
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("Slash commands registered!");

                // Start background update loop
                let ctx_clone = ctx.clone();
                let brawl_token_clone = brawl_token.clone();
                tokio::spawn(async move {
                    leaderboard::start_update_loop(ctx_clone, brawl_token_clone).await;
                });

                Ok(())
            })
        })
        .build();

    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .await?;

    println!("Bot is connected and running!");
    client.start().await.map_err(BotError::from)?;

    Ok(())
}
