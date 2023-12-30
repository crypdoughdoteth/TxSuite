use std::{convert::Infallible, fmt::Display, sync::OnceLock};

use sqlx::{migrate, FromRow, MySql, MySqlPool, Pool};

pub static RELATIONAL_DATABASE: OnceLock<Pool<MySql>> = OnceLock::new();

pub struct ProjectDatabases;

impl ProjectDatabases {
    pub async fn init(test: Option<()>) -> Result<(), Infallible> {
        // this is so we don't need to do system IO or alloc on every request
        let pool = match test {
            Some(_) => MySqlPool::connect(&dotenvy::var("TESTING_DATABASE_URL").unwrap())
                .await
                .unwrap(),
            None => MySqlPool::connect(&dotenvy::var("DATABASE_URL").unwrap())
                .await
                .unwrap(),
        };
        migrate!("./migrations").run(&pool).await.unwrap();
        RELATIONAL_DATABASE.get_or_init(|| pool);
        Ok(())
    }
}

#[derive(FromRow, Debug)]
pub struct Users {
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(FromRow, Debug)]
pub struct Sui {
    pub user_email: String,
    pub object: String,
}

#[derive(FromRow, Debug)]
pub struct Api {
    pub user_email: String,
    pub api_key: String,
}

#[derive(Debug)]
pub enum Roles {
    User,
    Partner,
    Admin,
}

impl From<String> for Roles {
    fn from(input: String) -> Roles {
        match input.as_ref() {
            "user" => Roles::User,
            "partner" => Roles::Partner,
            "admin" => Roles::Admin,
            _ => Roles::User,
        }
    }
}

impl Display for Roles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Roles::User => write!(f, "user"),
            Roles::Partner => write!(f, "partner"),
            Roles::Admin => write!(f, "admin"),
        }
    }
}
