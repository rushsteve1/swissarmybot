PRAGMA foreign_keys = ON;

CREATE TABLE quotes (
       id INTEGER PRIMARY KEY NOT NULL,
       text TEXT NOT NULL,
       user_id TEXT NOT NULL,
       user_name TEXT NOT NULL,
       author_id TEXT NOT NULL,
       author_name TEXT NOT NULL,
       inserted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
