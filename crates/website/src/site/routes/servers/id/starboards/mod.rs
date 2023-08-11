pub mod id;

use std::collections::HashMap;

use leptos::*;
use leptos_router::*;

use database::Starboard;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::site::components::{Card, CardList, CardSkeleton, ToastedSusp};

use super::{get_flat_guild, GuildIdContext};

pub type StarboardsResource =
    Resource<Option<Id<GuildMarker>>, Result<HashMap<i32, Starboard>, ServerFnError>>;

pub fn get_starboard(cx: Scope, starboard_id: i32) -> Option<Starboard> {
    let starboards = expect_context::<StarboardsResource>(cx);
    starboards
        .with(cx, |sbs| {
            sbs.as_ref().ok().map(|sbs| sbs.get(&starboard_id).cloned())
        })
        .flatten()
        .flatten()
}

#[server(GetStarboards, "/api")]
pub async fn get_starboards(
    cx: Scope,
    guild_id: u64,
) -> Result<HashMap<i32, Starboard>, ServerFnError> {
    use super::can_manage_guild;

    can_manage_guild(cx, guild_id).await?;

    let db = crate::db(cx);

    Starboard::list_by_guild(&db, guild_id as i64)
        .await
        .map_err(|e| e.into())
        .map(|v| v.into_iter().map(|s| (s.id, s)).collect())
}

#[component]
pub fn Starboards(cx: Scope) -> impl IntoView {
    let guild_id = expect_context::<GuildIdContext>(cx);
    let starboards: StarboardsResource = create_resource(
        cx,
        move || guild_id.get(),
        move |guild_id| async move {
            let Some(guild_id) = guild_id else {
                return Err(ServerFnError::ServerError("No guild ID.".to_string()));
            };
            get_starboards(cx, guild_id.get()).await
        },
    );
    provide_context(cx, starboards);

    let starboards_view = move |cx| {
        let guild = get_flat_guild(cx);
        let channel = move |id: Id<ChannelMarker>| {
            let guild = guild.clone();
            match guild {
                None => "unknown channel".to_string(),
                Some(g) => match g.channels.get(&id) {
                    None => "deleted channel".to_string(),
                    Some(c) => match &c.name {
                        None => "unknown channel".to_string(),
                        Some(n) => format!("#{n}"),
                    },
                },
            }
        };
        let title = move |sb: &Starboard| {
            format!("'{}' in {}", sb.name, channel(Id::new(sb.channel_id as _)))
        };
        let title = store_value(cx, title);
        starboards.read(cx).map(|sb| sb.map(|sb| {
            let sb = store_value(cx, sb);
            view! {cx,
                <For
                    each=move || sb.with_value(|sb| sb.clone())
                    key=|sb| sb.0
                    view=move |cx, sb| view! {cx, <Card title=title.with_value(|f| f(&sb.1)) href=sb.0.to_string()/>}
                />
            }
        }).map_err(|e| e.clone()))
    };

    view! {
        cx,
        <Outlet />
        <CardList>
            <ToastedSusp fallback=move || view! {cx,
                <For
                    each=|| 0..10
                    key=|t| *t
                    view=move |_, _| view!{cx, <CardSkeleton/>}
                />
            }>
                {move || starboards_view(cx)}
            </ToastedSusp>
        </CardList>
    }
}
