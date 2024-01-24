use std::collections::HashMap;

use maud::{html, Markup, DOCTYPE};

use crate::shared::{BigMoji, Drunk, Quote};

fn imgs_map() -> HashMap<&'static str, &'static str> {
	vec![
		("created with EMACS", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFgAAAAfBAMAAACR5hfTAAAAGFBMVEULCQi9ubj////AKSlIPDd4amGvmJW6a2bw4qAMAAACGUlEQVR4AbXUR3PzLBAA4NU3oFy1nwW6pv4ADylXyQPKFTIqVzOJ5LPHRX//ZbH19l5WhfYMLLjA9c/H5a9ggGt8Hy1+Pz7Bb+lPYOmo6hHX1UdDWXwnp3dGlYjXwTVN4zBvPsKxLqpzgyqEJUPBrv67NihylOBsGh6WQSjWrMrGVJgSuE4jFina1fX/17f+7RYzV5kMktQgFTx3ouWZcCZ3JuLWCbh6ub6r+SrHzFd1hpjWGIu1l9qa3NfhjnjlLL+UF5Zbw1GATwRDnSAVtkRkWZXFHUb80g0rYxBFP2j/o6Nb3Ta3tt36UWtWwbcjiTlfX7Wa9eNo0P/wQxlM3pb1czfan/m4rbG2MRfTR3gMzzE8Mq61OVItYgHAtHjuJ+tpWCm1ROVx8YSIr8vw2oWeQj2dcGu1HriZrEPEgkwR8C66R8QFLaHmNCTTuu72ra0wDC1PuCBcECZXPM4452Ca7sBtMuNJ+Q3hafEYOyinGYeZjdG6NKcEPU47P+22WGwDCiXtZHnGa2gokYY3MbkRi+Nu2iqPanx9ilnQ1PPMUFpbXvTPHM8ZbneEinAwM5Zq3iAwiqZJzjuZ/O4xoJ0nozzhYp4ZS1YCgC6T8/6ewtxS0TjhR4wHPuMBmGYBu9AZll48xiTpOGRM5mkT0poxlhAssFN2XvpYTHgKuXt6JTvjIaThB/eT/xtvuqbix/inA+AS/lG8A28KpFbhoqbWAAAAAElFTkSuQmCC"),
		("FIREFOX", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFgAAAAfCAMAAABUFvrSAAAAPFBMVEULR2/pjS7jmViPx9lhm7qUrb264etSgqYBAAHedRsFHEtHZ4bssX331av///uvMAz97n/6uSv94AK8ys5n1+OwAAACt0lEQVR4AbXT6ZriIBCFYYqlEYsYCPd/r3NOUkp0Yv/rD7LO4zvVm7v9Ue6WUEgWb7mwFzaO+x80jhdj/Hw2Rkr8V/Q4PrSMHQ6DIPfA4ubNstS6VNzsPd0E93FW8fQYiS2tteVBGh8m/CWw7/GZLgs/IUx7nyen0UgvONInHLG4cVR2ZvlYkw0JLNgdj4Qge6CsGhy5sF+Lp3pIpsLlkQBfFeiyZjLhCDO/VoS5V73Rb2HMy+BCySkdcG+E4R0HFjYfsPIhkbc8doIcbGHP+xFTCINfaKdbAecZPfsvsqdzXginNMIIgaS5H0XfG5bBV0HBnu128eJZSsGyka0UYqq9YwNWzdzEdLrFVPNnwiX6VM8l7ISRexcB7JxTlG0d2wamcq4UQXiZ4ovLyFgOnFXkCTM13fY+GhJusQthvpO6nGF+juj+C5VFi/Q+4ZPOdogbp3n4sux/s0kj4ZhzxFlVncajnH1fJ/yZqs1IS+b/UfyCataM4WzWGB17wuqbSG/9CkZFChjreMBCvlZiGJiKmx1s1HtpHfD2BSZRLvN0GW13im+VM7UGuF3CqvzKL10pFKwJ353ydYbvt61fw2gVdMWKeD3BaiqWgt1bN8jfYAUrVyyaMFKwZitgxS6/TKwGX9ndYCuT5ro7deh3GBOXT3hld1WfDbYyUGwLEwvg9v17vE7VWLr8YjLkD3emqtK3vhn8YA4L2Xl9sGJnoDwcVbhciBeF5iYu6jrCz++AHSNtZw9ICnrsB1S6d5cnbNf3ibULJpYJI/c2MWBksM27FpuY5nNn1Rd7u6vvRUS2i4m5nxPLE7aJy0r4szmxU+ebDXz5rWArSMKMKo9SVM+0vsE3wCqty4T/j+PNVquUSasW8d7jyRHFQuoI919gvYDLejwow42Ir60p2Tlxb01k6wf8R/0DD3hLM5DuCgEAAAAASUVORK5CYII="),
		("Wii", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFgAAAAfBAMAAACR5hfTAAAAElBMVEX7+/vq6urFxcWZmZl9fX0wMDCyfbTNAAABZUlEQVR4AYXM13HcMBSG0XOhAnSxaoCiCnDovwSnAhxYgRcaPy+hSM4mjr+5eMKZv9gqslTHedjG/VF3nNjG43AXUZ+zto2FzI6skYhNvNaI3tt/lyV67/sm0c9xpBiOcdSI5JFAoYx4fUMZld0xJqzDClEpdaDmEAQQJEhVJih0KaTQpk53WlZVJBRmCcLU/3o+EABN2vcGBQtO8/Ryp4W9aBYcDArNaX3hLa14T6pw8/H5Vrtgg1yxYE92gnK+rPxjxVPwu8mZcFn2GXJdhuq8BvT90TIFg+3EyrpA9rYNCRTMhtkUpmsIIrOODxSQXStpA0dvrWRfMKQt3IFj3NoWTrAsT5rZ7Li0lop5XW4mXUc/3m3AbjDr6/LhK77+4PDd4RvQF3x324SAYqNleXfrUUfbxm4dELtBK4Jf0za+yUii3juqjBs9DFHHz59Rc7wfx904Ztldt4OXj6q+dP/8xmf8BBIBd7xkj08BAAAAAElFTkSuQmCC"),
		("Made on Amiga", "data:image/gif;base64,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"),
		("spaceamp", "data:image/gif;base64,R0lGODlhWAAfAPIEAGuMjAAAAP///8zMzHuMjAAAAAAAAAAAACH/C05FVFNDQVBFMi4wAwEAAAAh+QQFFAAEACwAAAAAWAAfAAAD/0ix3P4wyklrVGLozbv/YCiO5LcIgAOUbEumzQozsiqcq6APge7/wKBwSCzmdr1jJjnw8W4LgHQ6s1ivEuo0oJVyuwzew0UubyBiR7oh7rrf8Lh8To9j7/i83szv+zl6gYKDMSlehIiJXocaAQMwdVFRdZR0XF9aUTyNjpuGlpehlaNuEVRcm2KNj5OHYJinsTSkdrBbt6kLqzOQn7elrqe2lLbDwpqdup5frbCtmb1gwLWYxtK5yaxvodbBwNyTkuLCkSvJ2b6vc93e27yx5di77LTq9ZWo5/P3/P3u8k9YJRqoJ0cPMTuaFFnIsKHDhwl1nEj4p6JFDz4mQizCAFpKj44gT4C84XEjxx4miaAk+ZFlxpcfRZJMKURjBj8lW+rMkNPlzj8ZD15cFabomTO6HClVVhGKBn/mkh5dSnWTUqsb/HGCOqVDVz5ctxQjSLagFwVl0xIikAAAIfkEBRQAAgAsBwALAEoACAAAAlIMfoaK7cmenLQqEzLW687YVeLYYFC3Jc4GAhz1fhGYlUyNujnt4mvdWuVIPVnvBkGeirpHyhM0jpAtqi9oSjaJ3MXWBuTNmDhfl2suhdQsi6IAACH5BAUUAAIALAgACwBJAAgAAAJAhBGpce0PX4q0osZctlzfDh6TFG7UtITcSJ4LC5mGrGo0FsG4qHs9ojitYkJRJzgD7lqV2433a0aBryczVcuCCgAh+QQFFAACACwHAAsASgAIAAACQAyOqcsZb5qTFMWK15tyX71lX9Ypo2kx4YkeLARRbzPW31xbLBDrZavhzRwhGsklAwZ3LhjvCDU2o9QqtGdlFAAAOw=="),
		("Y2K Compliant", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAXcAAACIAQMAAAAmvTnFAAAABlBMVEUhHwTs3R8dI2c7AAACt0lEQVR4Ae3VN478NhTH8R/Bgp35P8BgdAV3TvLqSi5pYLDkYHtfwOEs3MqHcHrOpenMQtDz40ibFTiAu52vQvVpJCa8ki7paOE5YAd0AyQf4CM6QpPmvKL9ye/FMwDFEUzwCV2e9+3JXxUfAM2kOIEzfI+5UgsHwcC7Joq/JUUJycEd5r07eS9eizdH0jEpckjtrHeTD6OHEv+RoneXfJtO/t+Xfrfm/7zzesM3VLz6MaK1xZu44SP8APV9HP8nbFj3dvS3JB5Ss+FNwAHQkfBhD6nDpnfiQ8K7LaSrKm9Y/BWkQ6XPeLeBlO99u+Jt8RaASlAxgdyGx0F8ADRB/Z6QM9xhyXsOe7RlflbM50dej351vUz+CnvxNPouoIloCK+lS3xWw8Vf/LpPXW85aeYjxxqfi4+q+FDlm9xwQPGqxvfFA56Pg67yNneTNzV+sKljTR0f+zpvkpdHfLY1ng3d+abKa+LBFp+6Kn8T5VU8VfrbyUdf5W+P8iIvnuv8jbyi+GOdJ10881FVeiOvIB51PhWvmEOtt8yDLt5X+XzvuzrfMPdG/Jt6n634t5tqnxrx7kzf22pPXfGm2kfPfBzqfeDi9Xme6706eVXrB6ATfzzTx3O9P8/Tub67nHcX///6S68kk4AByMjwGSZAEXSADTt4ji9910NzGD1P3qx4P8AwIaviw+QjbFzw1x72S0Iv/mOOD552uP53xmcLqxJ6neH06AWfPAgvSxZ7JAzPfVryZvTmkd8nWLfkcfK/Gffgd9v+1+LN5FsH2274txw8hwe/X/c/flI8Ru/abf+F+OGRtxv+aweHyacdrPls1X/fP/I8rHgz+vzM637eZ4tGJdAjr5hh1YI/NLA3BEqPPNll7wdYFv9m9JwVlWG2WPBdLyiC8Nwf5r1NUIwn3gQTJ/86uvQfJWQ1gEV2314AAAAASUVORK5CYII="),
		("Stand with Gooper", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFgAAAAfCAMAAABUFvrSAAAAPFBMVEVHcEzIzM5caXiVra9zkKT9//qFSkDl5eIREwmT2vAWFzNrrNFfCAjirpHJdlk8TVQjPyxTrVVAKLYeEt4jr9i+AAAAAXRSTlMAQObYZgAABBJJREFUeAF9lomioyAMRV8WkqagWP3/f52b0D5nP2BYPd7Z5+v54QGyYnp8t5t7uybof9w47ktfX19T3xy2OLSQfLimXA/XjhaYvxv26xD1vqX8fH6L+7QFNgRUKWmR6xBA6zAv3c6+FHVdwExxB4EH4t32fTfuEUIUJDaG5RGUQUnUg5UOI0KxYTq4d12SOTTyZS7x26zwos3JChNO8TYZE0YCt1miB6XYxy5DZUjXUW6MeBeySoxrpWbbgZlB3cuiJqIIPoTiNkdI7xB72tHFAqQTxRiytzilKzGY0CJ3/XSQViwMSu6yBQqgvK0Quxusphb5fR6ciVlvMaQAK4bwnZthCbdhWW0g6hiDnFEV94eVWIeoQRo20EcPHcP4Fr8bGyNtwpM1oCYbipiDAtMN4g3VAu+rWeDEVIaSh4xQ0zoYHEuMccF7wbaR++a0Qe6wtW1IwGHw05B8EUmNHdhoY3iJ0SPtlsJ3YoDDfd/QeEOqzZNtN4bNaZgym2CKfFZi7LUUWzPDu5SJJWAXHLTGK7E3NIbWDEUz7zIPM9NgY+o2bGgg7zDkGAYl3sLlTTGpxBqxDTggrsTREteUAsVCtvoWIltThUtITE0Ry8ZAEjUEAyRNCFfVMr03GTj/JG6FC6SOnvdaiWGW5qzUVbtE0mPh7Rfc6z5YtcSzZgiRXjRXfAqxl7nECmMqz+viAPD86l3imzuxM2+LRsre9tI2xweWWDvkZ/o9W/ude+dOnJ8P23lfiZW4YSgwNuldQ1V6nBNiXP5FXOmr3/hKnCey7fk3GxL73sTlW7wpr8Ac/UJqBxFRQoqMpNjEChVzCtjAEgPdF2ZQueYHqm+0lZMVqSFW8tATM5guPTEohtOb8kUul4YX8yOW/QPMtpHDuMOMFXc4mQNi7tcZ3k+ogvSSXEWtXM+L81vNoY53YkyFv81QAoc3tSnmYGGCqQvCVcbr9eJzXqdjN3KT1R12VNjuxKH2CQyxQ5v2Oee2E/dZfycRBHGdhNf5PF8vvWYFfycWAqdGmoMqccA8YUQrPZSUYoYYE+vCzBIRM1VOOfTX64WBX6IX50+18KnUz07lXYnJSaCEdpZ7mRH4mIbZPuFFYAm8eKog6znnq4aq2Myw7MEXBwXRJzGV2PaZdcJsqYb4MEztmJND8hZavBbwL/jq9z+zEriSZOKJsaLaqJp2mJEbkyO1SiKEQiBdCHozL/58VLJAjSWnWEQeBsZ4F/QDZd+PB2bPqaIieJKjtEc9VQ75GyWefAzweKCs8QG1PTDJpT2OyW/ws4NeT/HIBf/JTPFzzlKkqFqNtcZkTfHx6jBly3JPcPJpYNUnxHgeNxX4b+T/9B5/5XlURfj7v41YfH0938a/K+9tiP8LbN9iBP4BlvdV2q8uSucAAAAASUVORK5CYII="),
	].into_iter().collect()
}

pub fn header() -> Markup {
	html! {
		header {
			nav {
				ul {
					li { a href="/" { h3 style="margin: 0" { "SwissArmyBot" } } }
				}
				ul {
					li { a href="/bigmoji" { "BigMoji" } }
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

pub fn base(child: Markup) -> Markup {
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
			@for img in imgs_map() {
				img src=(img.1) alt=(img.0) title=(img.0) width="88" height="31";
			}
		}
	}
}

pub fn bigmoji(bigmoji: Vec<BigMoji>) -> Markup {
	html! {
		h1 { "BigMoji List" }

		div style="display: flex; justify-content: space-between;" {
			div { "There are " (bigmoji.len()) " BigMoji total" }
			div { kbd { "Ctrl-F" } " to search" }
		}

		table {
			thead {
				th { "Name" }
				th { "Text" }
				th { "Added at" }
			}
			tbody {
				@for moji in bigmoji {
					tr {
						td { code { (moji.name) } }
						td { (linkify(moji.text)) }
						td nowrap { (moji.inserted_at) }
					}
				}
			}
		}
	}
}

pub fn quotes(
	quotes: Vec<Quote>,
	selected: Option<u64>,
	from_date: String,
	to_date: String,
) -> Markup {
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
						td { strong { (drunk.score) } }
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

fn linkify(text: String) -> String {
	let text = text.trim();
	if text.starts_with("http") && !text.contains([' ', '\n']) {
		format!("<a href=\"{}\" target=\"_blank\">{}</a>", text, text)
	} else {
		text.to_string()
	}
}
