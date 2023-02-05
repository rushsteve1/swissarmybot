PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS quotes (
       id INTEGER PRIMARY KEY NOT NULL,
       text TEXT NOT NULL,
       user_id INTEGER NOT NULL,
       user_name TEXT NOT NULL,
       author_id INTEGER NOT NULL,
       author_name TEXT NOT NULL,
       inserted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS bigmoji (
       name TEXT PRIMARY KEY NOT NULL,
       text TEXT NOT NULL,
       inserted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
