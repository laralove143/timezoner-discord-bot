use anyhow::{bail, Result};
use chrono_tz::{Tz, TZ_VARIANTS};
use sqlx::SqlitePool;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::application_command::{CommandDataOption, CommandOptionValue},
    },
    id::UserId,
};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};

use crate::database;

pub async fn run(
    db: &SqlitePool,
    user_id: UserId,
    options: Vec<CommandDataOption>,
) -> Result<String> {
    let tz = if let CommandOptionValue::String(name) = &options[0].value {
        match get_timezone(name) {
            Some(tz) => tz,
            None => return Ok("i couldn't find that timezone >.< if you're sure it's right see my profile to report please".to_string()),
        }
    } else {
        bail!("first option for set_timezone is not string: {:?}", options);
    };

    database::set_timezone(db, user_id, tz).await?;

    Ok("tada! now you can use the `/time` command ^^".to_string())
}

fn get_timezone(name: &str) -> Option<&Tz> {
    TZ_VARIANTS.iter().find(|tz| name == tz.name())
}

pub fn build() -> Command {
    CommandBuilder::new(
        "set_timezone".to_string(),
        "set your time zone so that you can actually use the `/time` command".to_string(),
        CommandType::ChatInput,
    )
    .option(
        StringBuilder::new(
            "timezone".to_string(),
            "see my profile to learn what to put here.. sorry for the inconvenience >.<"
                .to_string(),
        )
        .required(true),
    )
    .build()
}
