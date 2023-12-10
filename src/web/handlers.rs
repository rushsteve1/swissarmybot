use askama_gotham::respond;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_empty_response;
use gotham::hyper::StatusCode;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use scraper::{html::Html, selector::Selector};
use serenity::all::{ChannelId, CreateMessage, Mentionable, Message};

use super::templates::*;
use crate::models::*;

use crate::{DB_POOL, GIT_VERSION, HTTP, VERSION};

const GOOD_STONKS: &str = "📈";
const BAD_STONKS: &str = "📉";
const STONKS_URL: &str = "https://finance.yahoo.com";
const STONKS_SEL: &str = "#marketsummary-itm-2 > h3:nth-child(1) > div:nth-child(4) > fin-streamer:nth-child(1) > span:nth-child(1)";

// TODO better error handling

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QuotesQuery {
    user: Option<i64>,
    from_date: Option<String>,
    to_date: Option<String>,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct ChannelQuery {
    channel: u64,
}

pub fn router() -> Router {
    build_simple_router(|route| {
        route.get("/").to(index);
        route.get("/bigmoji").to_async(bigmoji);
        route.get("/bigmoji.csv").to_async(bigmoji_csv);
        route
            .get("/quotes")
            .with_query_string_extractor::<QuotesQuery>()
            .to_async(quotes);
        route
            .get("/quotes.csv")
            .with_query_string_extractor::<QuotesQuery>()
            .to_async(quotes_csv);
        route
            .post("/random")
            .with_query_string_extractor::<ChannelQuery>()
            .to_async(post_random);
        route
            .post("/stonks")
            .with_query_string_extractor::<ChannelQuery>()
            .to_async(post_stonks);
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

async fn bigmoji_csv(state: State) -> HandlerResult {
    let bigmoji = get_bigmoji().await;

    let bigmoji = bigmoji
        .into_iter()
        .map(|mut b| {
            b.text = b.text.escape_default().collect();
            b
        })
        .collect();

    let tpl = BigMojiCSVTemplate { bigmoji };

    Ok((state, respond(&tpl)))
}

async fn quotes(mut state: State) -> HandlerResult {
    let query = QuotesQuery::take_from(&mut state);

    let (quotes, selected, from_date, to_date) = get_quotes(query).await;

    let tpl = QuotesTemplate {
        quotes,
        selected,
        from_date,
        to_date,
    };

    Ok((state, respond(&tpl)))
}

async fn quotes_csv(mut state: State) -> HandlerResult {
    let quotes = QuotesQuery::take_from(&mut state);

    let (quotes, _, _, _) = get_quotes(quotes).await;

    let quotes = quotes
        .into_iter()
        .map(|mut q| {
            q.text = q.text.escape_default().collect();
            q
        })
        .collect();

    let tpl = QuotesCSVTemplate { quotes };

    Ok((state, respond(&tpl)))
}

async fn post_random(mut state: State) -> HandlerResult {
    let chan = ChannelQuery::take_from(&mut state);
    let body = gotham::hyper::Body::take_from(&mut state);
    let chan: ChannelId = chan.channel.into();

    let body = gotham::hyper::body::to_bytes(body)
        .await
        .ok()
        .and_then(|b| String::from_utf8(b.to_vec()).ok())
        .unwrap_or_default();

    post_random_to_channel(chan, body)
        .await
        .expect("Could not post random quote");

    let resp = create_empty_response(&state, StatusCode::ACCEPTED);
    Ok((state, resp))
}

pub async fn post_random_to_channel(
    chan: ChannelId,
    body: String,
) -> Result<Message, serenity::Error> {
    let quote = get_random_quote().await;
    let user_id = serenity::model::id::UserId::new(quote.user_id as u64);
    let author_id = serenity::model::id::UserId::new(quote.author_id as u64);

    let txt = format!(
        "{}\n#{} by {}, added by {} on <t:{}:f>\n\n>>> {}",
        body,
        quote.id,
        user_id.mention(),
        author_id.mention(),
        quote.inserted_at.timestamp(),
        quote.text
    );

    chan.send_message(HTTP.get().unwrap(), CreateMessage::new().content(txt))
        .await
}

async fn post_stonks(mut state: State) -> HandlerResult {
    let chan = ChannelQuery::take_from(&mut state);
    let chan: ChannelId = chan.channel.into();

    post_stonks_to_channel(chan)
        .await
        .expect("Could not post stonks message");

    let resp = create_empty_response(&state, StatusCode::ACCEPTED);
    Ok((state, resp))
}

pub async fn post_stonks_to_channel(chan: ChannelId) -> Result<Message, serenity::Error> {
    let txt = {
        let body = reqwest::get(STONKS_URL)
            .await
            .expect("Failed to get Yahoo Finance")
            .text()
            .await
            .expect("Could not get Stonks body");
        let document = Html::parse_document(&body);
        let selector = Selector::parse(STONKS_SEL).expect("Failed to parse selector");
        let el = document.select(&selector).next().unwrap();
        let c = el.text().next().unwrap().chars().next().unwrap();
        match c {
            '+' => GOOD_STONKS,
            '-' => BAD_STONKS,
            _ => unreachable!(),
        }
    };

    chan.send_message(HTTP.get().unwrap(), CreateMessage::new().content(txt))
        .await
}

// --- Helper Functions ---

async fn get_bigmoji() -> Vec<BigMoji> {
    sqlx::query_as::<_, BigMoji>("SELECT * FROM bigmoji;")
        .fetch_all(&*DB_POOL)
        .await
        .expect("Error getting bigmoji")
}

async fn get_quotes(query: QuotesQuery) -> (Vec<Quote>, i64, String, String) {
    let from_date = query
        .from_date
        .clone()
        .unwrap_or_else(|| "1970-01-01".into());
    let to_date = query.to_date.clone().unwrap_or_else(|| "3000-01-01".into());
    let user_id = query.user.unwrap_or(0);

    let quotes = if user_id > 0 {
        sqlx::query_as::<_, Quote>(
            "SELECT * FROM quotes WHERE user_id = ? AND inserted_at BETWEEN ? AND ?;",
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(&*DB_POOL)
        .await
        .expect("Error getting quotes")
    } else {
        sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE inserted_at BETWEEN ? AND ?;")
            .bind(from_date)
            .bind(to_date)
            .fetch_all(&*DB_POOL)
            .await
            .expect("Error getting quotes")
    };

    (
        quotes,
        user_id,
        query.from_date.unwrap_or_default(),
        query.to_date.unwrap_or_default(),
    )
}

async fn get_random_quote() -> Quote {
    sqlx::query_as::<_, Quote>("SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(&*DB_POOL)
        .await
        .expect("Error getting quote")
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::hyper::StatusCode;
    use gotham::test::TestServer;

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
    fn get_bigmoji_csv() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/bigmoji.csv")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.read_utf8_body().unwrap();
        assert!(body.starts_with("name,text,inserted_at"));
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

    #[test]
    fn get_quotes_csv() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/quotes.csv")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.read_utf8_body().unwrap();
        assert!(body.starts_with("id,user_id,user_name,author_id,author_name,text,inserted_at"));
    }
}
