//! a discord bot that makes timezone conversions really easy

#![warn(clippy::cargo, clippy::nursery, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::shadow_same,
    clippy::pattern_type_mismatch
)]

/// functions to set up, update and retrieve timezone information from the
/// sqlite database
mod database;
/// functions to handle events
mod event;
/// functions to create and handle interaction
mod interaction;
/// functions to parse time from strings and format them into discord's
/// epoch formatting
mod parse;

use std::{env, path::Path, sync::Arc};

use aes_gcm_siv::{aead::NewAead, Aes128GcmSiv};
use anyhow::{IntoResult, Result};
use futures::StreamExt;
use sqlx::SqlitePool;
use tantivy::{Index, IndexReader};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_error::ErrorHandler;
use twilight_gateway::{Cluster, EventTypeFlags, Intents};
use twilight_http::Client;
use twilight_model::id::{
    marker::{ApplicationMarker, UserMarker},
    Id,
};
use twilight_webhook::cache::Cache as WebhookCache;

/// arced context data for thread safety
type Context = Arc<ContextValue>;

/// inner data of the context
pub struct ContextValue {
    /// used to make http requests to discord
    http: Client,
    /// used to check permissions and channels
    cache: InMemoryCache,
    /// used to impersonate message authors
    webhooks: WebhookCache,
    /// used to handle errors
    error_handler: ErrorHandler,
    /// used for the user's timezone information
    db: SqlitePool,
    /// used to encrypt the database
    cipher: Aes128GcmSiv,
    /// used for creating the interaction client
    application_id: Id<ApplicationMarker>,
    /// used for permissions cache
    user_id: Id<UserMarker>,
    /// used for timezone autocomplete
    searcher: (Index, IndexReader),
}

#[tokio::main]
async fn main() -> Result<()> {
    let intents = Intents::GUILDS
        | Intents::GUILD_WEBHOOKS
        | Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT;
    let event_types = EventTypeFlags::INTERACTION_CREATE
        | EventTypeFlags::WEBHOOKS_UPDATE
        | EventTypeFlags::MESSAGE_CREATE
        | EventTypeFlags::GUILD_CREATE
        | EventTypeFlags::GUILD_UPDATE
        | EventTypeFlags::GUILD_DELETE
        | EventTypeFlags::ROLE_CREATE
        | EventTypeFlags::ROLE_UPDATE
        | EventTypeFlags::ROLE_DELETE
        | EventTypeFlags::CHANNEL_CREATE
        | EventTypeFlags::CHANNEL_UPDATE
        | EventTypeFlags::CHANNEL_DELETE
        | EventTypeFlags::THREAD_CREATE
        | EventTypeFlags::THREAD_DELETE
        | EventTypeFlags::THREAD_UPDATE
        | EventTypeFlags::THREAD_LIST_SYNC
        | EventTypeFlags::THREAD_MEMBER_UPDATE
        | EventTypeFlags::THREAD_MEMBERS_UPDATE
        | EventTypeFlags::MEMBER_ADD
        | EventTypeFlags::MEMBER_CHUNK
        | EventTypeFlags::MEMBER_UPDATE
        | EventTypeFlags::MEMBER_REMOVE;
    let resource_types =
        ResourceType::GUILD | ResourceType::CHANNEL | ResourceType::MEMBER | ResourceType::ROLE;

    let token = env::var("TIMEZONER_BOT_TOKEN")?;

    let (cluster, mut events) = Cluster::builder(token.clone(), intents)
        .event_types(event_types)
        .build()
        .await?;
    let cluster_spawn = Arc::new(cluster);
    tokio::spawn(async move { cluster_spawn.up().await });

    let http = Client::new(token);
    let application_id = http
        .current_user_application()
        .exec()
        .await?
        .model()
        .await?
        .id;
    let user_id = http.current_user().exec().await?.model().await?.id;

    interaction::create(&http, application_id).await?;

    let db = database::new().await?;

    let cipher = Aes128GcmSiv::new_from_slice(&hex::decode(env::var("KEY")?)?)?;

    let cache = InMemoryCache::builder()
        .resource_types(resource_types)
        .build();
    let webhooks = WebhookCache::new();
    let mut error_handler = ErrorHandler::new();
    error_handler.channel(
        http.create_private_channel(
            http.current_user_application()
                .exec()
                .await?
                .model()
                .await?
                .owner
                .ok()?
                .id,
        )
        .exec()
        .await?
        .model()
        .await?
        .id,
    );
    error_handler.file("timezoner_errors.txt".into());

    let mut timezones_index = Index::open_in_dir(Path::new("timezones_index"))?;
    timezones_index.set_default_multithread_executor()?;
    let timezones_reader = timezones_index.reader()?;
    let searcher = (timezones_index, timezones_reader);

    let ctx = Arc::new(ContextValue {
        http,
        cache,
        webhooks,
        error_handler,
        db,
        cipher,
        application_id,
        user_id,
        searcher,
    });

    while let Some((_, event)) = events.next().await {
        ctx.cache.update(&event);
        tokio::spawn(event::handle(Arc::clone(&ctx), event));
    }

    Ok(())
}
