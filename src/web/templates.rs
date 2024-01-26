use include_data_uri::include_data_uri;
use maud::{html, Markup, DOCTYPE};

use crate::shared::{Drunk, Quote};

const fn imgs_map() -> &'static [(&'static str, &'static str)] {
	&[
		("created with EMACS", include_data_uri!("imgs/emacs.png")),
		("FIREFOX", include_data_uri!("imgs/firefox.png")),
		("Wii", include_data_uri!("imgs/wii.png")),
		("Made on Amiga", include_data_uri!("imgs/amiga.gif")),
		("spaceamp", include_data_uri!("imgs/spaceamp.gif")),
		("Y2K Compliant", include_data_uri!("imgs/y2k.png")),
		("Stand with Gooper", include_data_uri!("imgs/gooper.png")),
	]
}

pub fn header() -> Markup {
	html! {
		header {
			nav {
				ul {
					li { a href="/" { h3 style="margin: 0" { "SwissArmyBot" } } }
				}
				ul {
					li { a href="/quotes" { "Quotes" } }
					li { a href="/drunks" { "Drunks" } }
				}
			}
		}
	}
}

pub fn footer() -> Markup {
	html! {
		footer style="text-align: center" {
			div { a href="#" { "Back to top" } }
			"Copyright © 2024 "
			a href="https://rushsteve1.us" target="_blank" { "rushsteve1" }
			" | View on "
			a href="https://github.com/rushsteve1/swissarmybot" target="_blank" { "GitHub" }
		}
	}
}

pub fn base(child: &Markup) -> Markup {
	html! {
		(DOCTYPE)
		html {
			head {
				meta charset="utf-8";
				meta name="viewport" content="width=device-width, initial-scale=1";
				title { "SwissArmyBot" }
				link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@next/css/pico.fluid.classless.min.css";
			}
			body {
				(header())
				main { (child) }
				(footer())
			}
		}
	}
}

pub fn index(version: &'static str, git_version: Option<&'static str>) -> Markup {
	html! {
		blockquote {
			"A Discord bot that does a whole bunch of things that no one needs. Like a Swiss Army Knife."
		}
		div {
			div {
				"Version "
				code { (version) }
			}
			div {
				"Git SHA: "
				code { (git_version.unwrap_or("Unknown")) }
			}
		}

		div {
			h3 { "Help" }
			p {
				"Type " kbd { "/" } " on Discord and follow the prompts"
			}
			p {
				r"Referencing quotes (for getting and removing them) is done with their ID
				number. ID numbers are unique across all of SAB, not just a single
				person like they used to be."
			}
			p {
				"All dates and times on this site are in UTC because I'm lazy."
			}
		}

		div style="display: flex; justify-content: space-around;" {
			@for (title, data) in imgs_map() {
				img src=(data) alt=(title) title=(title) width="88" height="31";
			}
		}
	}
}

pub fn quotes(quotes: Vec<Quote>, selected: Option<u64>, from_date: &str, to_date: &str) -> Markup {
	html! {
		h1 { "Quote List" }

		div style="display: flex; justify-content: space-between;" {
			div { "There are " (quotes.len()) " quotes total" }
			div { kbd { "Ctrl-F" } " to search" }
		}

		form method="GET" {
			fieldset role="group" {
				input type="number" id="user-id" name="user" readonly="true" value=(selected.unwrap_or_default());
				input type="date" name="from_date" value=(from_date);
				input type="date" name="to_date" value=(to_date);
				input type="submit" value="Submit";
				a href="/quotes" role="button" { "Clear" }
			}
		}

		table {
			thead {
				th { "ID" }
				th { "Text" }
				th { "User" }
				th { "Author" }
				th { "Added at" }
			}
			tbody {
				@for quote in quotes {
					tr {
						td { code { (quote.id) } }
						td width="99%" { (quote.text) }
						td nowrap title=(quote.user_id) {
							a href={"?user=" (quote.user_id) } { (quote.user_name)}
						}
						td nowrap title=(quote.user_id) { (quote.author_name) }
						td nowrap { (quote.inserted_at) }
					}
				}
			}
		}
	}
}

pub fn drunks(drunks: Vec<Drunk>, last_spill_days: i64) -> Markup {
	html! {
		h2 style="text-align: center" { u { (last_spill_days) } " days since last spill" }
		hr;

		h1 { "Drunks List" }

		div style="display: flex; justify-content: space-between;" {
			div { "There are " (drunks.len()) " drunkards on the leaderboard" }
			div { kbd { "Ctrl-F" } " to search" }
		}

		table {
			thead {
				th { "Drunkard" }
				th {
					abbr title="beer + (wine * 2) + (shots * 2) + (cocktails * 2) + (derby * 3)" {
						"Score"
					}
				}
				th { "Beer" }
				th { "Wine" }
				th { "Shots" }
				th { "Cocktails" }
				th { "Derby" }
				th { "Water" }
				th nowrap { "Last Drink" }
				th nowrap { "Last Drink At" }
				th nowrap { "Last Spill At" }
			}
			tbody {
				@for drunk in drunks {
					tr {
						td nowrap { (drunk.user_name) }
						td { strong { (drunk.score()) } }
						td { (drunk.beer) }
						td { (drunk.wine) }
						td { (drunk.shots) }
						td { (drunk.cocktails) }
						td { (drunk.derby) }
						td { (drunk.water) }
						td nowrap { (drunk.last_drink.clone().unwrap_or_default()) }
						td nowrap { (drunk.updated_at) }
						td nowrap { (drunk.last_spill_str()) }
					}
				}
			}
		}
	}
}
