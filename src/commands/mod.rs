mod bigmoji;
mod definition;
mod drunk;
mod handler;
mod quotes;

pub use bigmoji::BigMoji;
pub use drunk::Drunk;
pub use handler::Handler;
pub use quotes::Quote;

use serenity::model::application::CommandDataOptionValue as OptionValue;

trait AsInner {
    fn as_string(&self) -> Option<&String>;
    fn as_int(&self) -> Option<i64>;
    fn as_user(&self) -> Option<&serenity::model::id::UserId>;
}

impl AsInner for OptionValue {
    fn as_string(&self) -> Option<&String> {
        if let OptionValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    fn as_int(&self) -> Option<i64> {
        if let OptionValue::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    fn as_user(&self) -> Option<&serenity::model::id::UserId> {
        if let OptionValue::User(u) = self {
            Some(u)
        } else {
            None
        }
    }
}
