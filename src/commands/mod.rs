mod bigmoji;
mod definition;
mod handler;
mod quotes;

pub use handler::{get_cmd, Handler};

use serenity::model::application::interaction::application_command::CommandDataOptionValue as OptionValue;

trait AsInner {
    fn as_string(&self) -> Option<&String>;
    fn as_int(&self) -> Option<i64>;
    fn as_user(&self) -> Option<&serenity::model::user::User>;
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

    fn as_user(&self) -> Option<&serenity::model::user::User> {
        if let OptionValue::User(u, _) = self {
            Some(u)
        } else {
            None
        }
    }
}
