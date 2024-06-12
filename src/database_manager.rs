use crate::content_manager::{Account, Id, Post};
use rusqlite::{params, Connection, Result, Row};

pub struct DatabaseManager {
    connection: Connection,
}

// Worst 2 hours of my fucking life.

impl DatabaseManager {
    pub fn connect(link: &str) -> Self {
        Self {
            connection: Connection::open(link).expect("[FATAL] FAILED TO CONNECT TO DB"),
        }
    }

    pub fn query_get_posts(&self) -> Vec<Post> {
        let mut statement = self
            .connection
            .prepare(
                "
          SELECT id, title, content, author
          FROM posts
      ",
            )
            .expect("[FATAL] FAILED STATEMENT PREP IN GET_POSTS");

        let posts_result = statement
            .query_map(params![], |row| {
                let post_id: i32 = row.get(0)?;
                let title: String = row.get(1)?;
                let content: String = row.get(2)?;
                let author_id: String = row.get(3)?;

                Ok(Post::new(
                    post_id.to_string(),
                    title,
                    content,
                    author_id.to_string(),
                ))
            })
            .expect("[FATAL] FAILED SQL QUERY IN GET_POSTS");

        posts_result.map(|r| r.unwrap()).collect()
    }

    pub fn query_add_post(&mut self, title: String, content: String, author: Id) {
        self.connection
            .execute(
                "
          INSERT INTO posts (title, content, author)
          VALUES (?, ?, ?)
      ",
                params![title, content, author.as_str()],
            )
            .expect("[FATAL] FAILED SQL QUERY IN ADD_POST");
    }

    pub fn query_remove_post(&mut self, id: Id) {
        self.connection
            .execute(
                "
          DELETE FROM posts
          WHERE id = ?
      ",
                params![id.as_str()],
            )
            .expect("[FATAL] FAILED SQL QUERY IN REMOVE_POST");
    }

    pub fn query_get_account_info(&self, id: Id) -> Option<Account> {
        let mut statement = self
            .connection
            .prepare(
                "
          SELECT id, username, password, admin
          FROM accounts
          WHERE id = ?
      ",
            )
            .expect("[FATAL] FAILED STATEMENT PREP IN GET_ACCOUNT_INFO");

        let account_result = statement.query_row(params![id.as_str()], |row| {
            let account_id: i32 = row.get(0)?;
            let username: String = row.get(1)?;
            let password: String = row.get(2)?;
            let is_admin: i32 = row.get(3)?;

            Ok(Account::new(
                account_id.to_string(),
                username,
                password,
                is_admin,
            ))
        });

        match account_result {
            Ok(account) => Some(account),
            Err(_) => None,
        }
    }

    pub fn query_add_account(&mut self, username: String, password: String) {
        self.connection
            .execute(
                "
          INSERT INTO accounts (username, password)
          VALUES (?, ?)
      ",
                params![username, password],
            )
            .expect("[FATAL] FAILED SQL QUERY IN ADD_ACCOUNT");
    }
}
