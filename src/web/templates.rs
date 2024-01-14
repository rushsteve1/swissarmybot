use crate::shared::{BigMoji, Drunk, Quote};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
	pub version: &'static str,
	pub git_version: Option<&'static str>,
}

#[derive(Template)]
#[template(path = "bigmoji.html")]
pub struct BigMojiTemplate {
	pub bigmoji: Vec<BigMoji>,
}

#[derive(Template)]
#[template(path = "quotes.html")]
pub struct QuotesTemplate {
	pub quotes: Vec<Quote>,
	pub selected: Option<u64>,
	pub from_date: String,
	pub to_date: String,
}

#[derive(Template)]
#[template(path = "drunks.html")]
pub struct DrunksTemplate {
	pub drunks: Vec<Drunk>,
	pub last_spill_days: i64,
}

mod filters {
	pub fn env(name: &str) -> askama::Result<String> {
		Ok(std::env::var(name).unwrap_or_else(|_| String::new()))
	}

	pub fn linkify(text: &str) -> askama::Result<String> {
		let text = text.trim();
		if text.starts_with("http") && !text.contains([' ', '\n']) {
			Ok(format!(
				"<a href=\"{}\" target=\"_blank\">{}</a>",
				text, text
			))
		} else {
			Ok(text.to_string())
		}
	}
}
