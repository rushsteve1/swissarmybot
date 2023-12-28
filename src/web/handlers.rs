use askama_gotham::respond;
use gotham::handler::HandlerResult;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham_derive::StateData;
use gotham_derive::StaticResponseExtender;
use serde::Deserialize;

use super::templates::*;

use crate::helpers::get_bigmoji;
use crate::helpers::get_drunks;
use crate::helpers::get_quotes;
use crate::{GIT_VERSION, VERSION};

// TODO better error handling

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QuotesQuery {
    user: Option<i64>,
    from_date: Option<String>,
    to_date: Option<String>,
}

pub fn router() -> Router {
    build_simple_router(|route| {
        route.get("/").to(index);
        route.get("/bigmoji").to_async(bigmoji);
        route
            .get("/quotes")
            .with_query_string_extractor::<QuotesQuery>()
            .to_async(quotes);
        route.get("/drunks").to_async(drunks)
    })
}

fn index(state: State) -> (State, IndexTemplate) {
    let tpl = IndexTemplate {
        version: VERSION,
        git_version: GIT_VERSION,
    };

    (state, tpl)
}

async fn bigmoji(state: State) -> HandlerResult {
    let bigmoji = get_bigmoji().await;

    let tpl = BigMojiTemplate { bigmoji };

    Ok((state, respond(&tpl)))
}

async fn quotes(mut state: State) -> HandlerResult {
    let query = QuotesQuery::take_from(&mut state);
    let from_date = query
        .from_date
        .clone()
        .unwrap_or_else(|| "1970-01-01".into());
    let to_date = query.to_date.clone().unwrap_or_else(|| "3000-01-01".into());
    let user_id = query.user.unwrap_or(0);

    let (quotes, selected, from_date, to_date) = get_quotes(from_date, to_date, user_id).await;

    let tpl = QuotesTemplate {
        quotes,
        selected,
        from_date,
        to_date,
    };

    Ok((state, respond(&tpl)))
}

async fn drunks(state: State) -> HandlerResult {
    let drunks = get_drunks().await;
    let tpl = DrunksTemplate { drunks };

    Ok((state, respond(&tpl)))
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use gotham::hyper::StatusCode;
    use gotham::test::TestServer;

    use crate::web::router;

    #[test]
    fn get_bigmoji() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/bigmoji")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.read_utf8_body().unwrap();
        assert!(body.contains("BigMoji List"));
    }

    #[test]
    fn get_quotes() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/quotes")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.read_utf8_body().unwrap();
        assert!(body.contains("Quotes List"));
    }
}
