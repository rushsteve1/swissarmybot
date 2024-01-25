mod bigmoji;
mod definition;
mod drunks;
mod handler;
mod quotes;

pub use handler::Handler;

use serenity::model::application::CommandDataOptionValue as OptionValue;

trait AsInner {
	fn as_string(&self) -> Option<&String>;
	fn as_int(&self) -> Option<i64>;
	fn as_user(&self) -> Option<&serenity::model::id::UserId>;
}

impl AsInner for OptionValue {
	fn as_string(&self) -> Option<&String> {
		if let Self::String(s) = self {
			Some(s)
		} else {
			None
		}
	}

	fn as_int(&self) -> Option<i64> {
		if let Self::Integer(i) = self {
			Some(*i)
		} else {
			None
		}
	}

	fn as_user(&self) -> Option<&serenity::model::id::UserId> {
		if let Self::User(u) = self {
			Some(u)
		} else {
			None
		}
	}
}
