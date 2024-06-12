use actix_web::web::post;

use crate::database_manager::DatabaseManager;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, PartialOrd)]
pub struct Post {
    id: Id,
    title: String,
    content: String,
    author: Id,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct IntermediatePost {
    title: String,
    content: String,
    author: i32,
}

impl Post {
    pub fn new(id: String, title: String, content: String, author: String) -> Self {
        Self {
            id: Id::PostId(id.parse::<i32>().unwrap()),
            title: title,
            content: content,
            author: Id::AccountId(author.parse::<i32>().unwrap()),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct Account {
    id: Id,
    username: String,
    password: String,
    is_admin: bool,
}

impl Account {
    pub fn new(id: String, username: String, password: String, is_admin: i32) -> Self {
        Self {
            id: Id::AccountId(id.parse::<i32>().unwrap()),
            username: username,
            password: password,
            is_admin: is_admin != 0,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, PartialOrd)]
#[serde(untagged)]
pub enum Id {
    AccountId(i32),
    PostId(i32),
}

impl Id {
    pub fn as_str(&self) -> String {
        match self {
            Id::AccountId(id) => format!("{}", id),
            Id::PostId(id) => format!("{}", id),
        }
    }
    pub fn as_int(&self) -> i32 {
        match self {
            Id::AccountId(id) => id.to_owned(),
            Id::PostId(id) => id.to_owned(),
        }
    }
    pub fn validate_post_id(self) -> Option<Self> {
        match self {
            Id::AccountId(_) => Option::None,
            Id::PostId(_) => Some(self),
        }
    }
    pub fn validate_account_id(self) -> Option<Self> {
        match self {
            Id::AccountId(_) => Some(self),
            Id::PostId(_) => Option::None,
        }
    }
}

pub struct ContentManager {
    db_manager: DatabaseManager,
}

pub enum DeletePostError {
    PostDoesntExist(Id),
    IndalidPostId(Id),
    AccountDoesntExist(Id),
    InvalidAccountId(Id),
    UnauthorizedDeletion,
}

impl ContentManager {
    pub fn new(db_manager: DatabaseManager) -> Self {
        Self {
            db_manager: db_manager,
        }
    }

    pub fn get_post(&self, id: Id) -> Option<Post> {
        if let Some(id) = id.validate_post_id() {
            let mut posts = self.db_manager.query_get_posts();
            if let Some(post) = posts.drain(..).find(|post| post.id == id) {
                return Some(post.clone());
            } else {
                return Option::None;
            };
        } else {
            Option::None
        }
    }

    pub fn get_posts(&self) -> Option<Vec<Post>> {
        let posts = self.db_manager.query_get_posts();
        &posts.iter().for_each(|ref p| println!("{:#?}", p));
        if posts.len() <= 0 {
            Option::None
        } else {
            Some(posts)
        }
    }

    pub fn create_post(&mut self, inter_post: IntermediatePost) -> Option<Post> {
        // this is the most hacky and shitty and un-thread safe fucking way of doing this but i am too lazy to implement it in a better way.

        let mut largest = 0;

        for post in self.db_manager.query_get_posts() {
            if post.id.as_int() > largest {
                largest = post.id.as_int();
            };
        }

        self.db_manager.query_add_post(
            inter_post.title,
            inter_post.content,
            Id::AccountId(inter_post.author),
        );

        if let Some(post) = self.get_post(Id::PostId(largest + 1)) {
            Some(post)
        } else {
            Option::None
        }
    }
    //decided i might as well add errors here cuz its easy...

    pub fn delete_post(&mut self, id_of_post_to_delete: Id, acc_id: Id) -> Option<DeletePostError> {
        let post_id: Id;
        let account_id: Id;
        let account: Account;
        let post_to_delete: Post;

        // guard satements...

        if let Some(ref valid_post_id) = id_of_post_to_delete.clone().validate_post_id() {
            post_id = valid_post_id.to_owned();
        } else {
            return Some(DeletePostError::IndalidPostId(id_of_post_to_delete));
        }
        if let Some(ref valid_account_id) = acc_id.clone().validate_account_id() {
            account_id = valid_account_id.to_owned();
        } else {
            return Some(DeletePostError::InvalidAccountId(acc_id));
        }
        if let Some(ref account_inf) = self.db_manager.query_get_account_info(account_id.clone()) {
            account = account_inf.to_owned();
        } else {
            return Some(DeletePostError::AccountDoesntExist(account_id));
        }
        if let Some(ref _post_to_delete) = self.get_post(post_id.clone()) {
            post_to_delete = _post_to_delete.to_owned();
        } else {
            return Some(DeletePostError::PostDoesntExist(post_id));
        }

        if post_to_delete.author != account_id.clone() && !account.is_admin {
            return Some(DeletePostError::UnauthorizedDeletion);
        } else {
            self.db_manager.query_remove_post(post_id);
            return Option::None;
        }
    }
}
