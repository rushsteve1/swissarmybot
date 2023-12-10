use serenity::all::Interaction;

use super::get_cmd;
use crate::models::BigMoji;
use crate::DB_POOL;

pub async fn add(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    let text = cmd.value.as_str().unwrap().to_string();

    if name.len() < 3 {
        return "BigMoji name too short".to_string();
    }

    // Prevents recursive BigMoji
    let text = text.replace(&format!(":{}:", name), "");

    sqlx::query("INSERT INTO bigmoji (name, text) VALUES (?, ?);")
        .bind(&name)
        .bind(text)
        .execute(&*DB_POOL)
        .await
        .expect("Error inserting bigmoji");

    format!("BigMoji `:{}:` added", name)
}

pub async fn remove(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    sqlx::query("DELETE FROM bigmoji WHERE name = ?;")
        .bind(&name)
        .execute(&*DB_POOL)
        .await
        .expect("Error deleting bigmoji");

    format!("Deleted BigMoji `:{}:`", name)
}

pub async fn get(interaction: &Interaction) -> String {
    let cmd = get_cmd(interaction);

    let mut name = cmd.name.replace(':', "").to_lowercase();
    name.retain(|c| !c.is_whitespace());

    let moji: Option<BigMoji> = sqlx::query_as("SELECT * FROM bigmoji WHERE name = ?;")
        .bind(&name)
        .fetch_optional(&*DB_POOL)
        .await
        .expect("Error getting bigmoji");

    if let Some(moji) = moji {
        moji.text
    } else {
        format!("BigMoji `:{}:` does not exist", name)
    }
}
