pub mod auth;
pub mod servers;
pub mod website;

use leptos::*;
use leptos_router::*;
use twilight_model::user::CurrentUser;

use super::errors;

#[server(GetUser, "/api")]
pub async fn get_user(cx: Scope) -> Result<CurrentUser, ServerFnError> {
    use crate::auth::context::AuthContext;
    let Some(acx) = AuthContext::get(cx) else {
        return Err(ServerFnError::ServerError("Unauthorized.".to_string()));
    };

    Ok(acx.http.current_user().await?.model().await?)
}

pub type UserRes = Resource<(), Result<CurrentUser, ServerFnError>>;

#[component]
pub fn Index(cx: Scope) -> impl IntoView {
    let user: UserRes = create_resource(cx, || (), move |_| get_user(cx));
    provide_context(cx, user);

    view! { cx,
        <Router>
            <Routes>
                <Route path="/auth/redirect" view=auth::redirect::AuthRedirect/>
                <Route path="/auth/login" view=auth::login::Login/>

                <WebsiteRoutes/>
                <DashboardRoutes/>
            </Routes>
        </Router>
    }
}

#[component(transparent)]
fn WebsiteRoutes(cx: Scope) -> impl IntoView {
    view! { cx,
        <Route path="" view=website::Website>
            <Route path="" view=website::home::Home/>

            <Route path="/*any" view=errors::not_found::NotFound/>
        </Route>
    }
}

#[component(transparent)]
fn DashboardRoutes(cx: Scope) -> impl IntoView {
    view! { cx,
        <Route path="/servers" view=servers::Servers>
            <Route path="" view=servers::server_list::ServerList/>
            <Route path=":id" view=servers::id::Server>
                <Route path="" view=servers::id::overview::Overview/>
            // <Route path="/starboards" view=dashboard::starboards::Starboards>
            // <Route path=":id" view=dashboard::starboards::Requirements/>
            // <Route path=":id/behavior" view=dashboard::starboards::Behavior/>
            </Route>

            <Route path="/*any" view=errors::not_found::NotFound/>
        </Route>
    }
}
