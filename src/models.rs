use chrono::NaiveDateTime;

#[derive(sqlx::FromRow)]
pub struct BigMoji {
    pub name: String,
    pub text: String,
    pub inserted_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub struct Quote {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub author_id: i64,
    pub author_name: String,
    pub text: String,
    pub inserted_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub struct Drunk {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub beer: i64,
    pub wine: i64,
    pub shots: i64,
    pub cocktails: i64,
    pub derby: i64,
    pub updated_at: NaiveDateTime,
}
