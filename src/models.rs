#[derive(sqlx::FromRow)]
pub struct BigMoji {
    pub name: String,
    pub text: String,
    pub inserted_at: chrono::DateTime<chrono::offset::Local>,
}

#[derive(sqlx::FromRow)]
pub struct Quote {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub author_id: i64,
    pub author_name: String,
    pub text: String,
    pub inserted_at: chrono::DateTime<chrono::offset::Local>,
}
