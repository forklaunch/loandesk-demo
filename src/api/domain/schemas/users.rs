use forklaunch_core::contract::datetime::{DateTime, Utc};

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[forklaunch_core::contract::schema]
#[derive(Clone)]
pub struct NewUser {
    #[field(max_length = 254)]
    pub email: String,
    #[field(max_length = 400)]
    pub full_name: String,
    #[field(max_length = 40)]
    pub role: String,
    #[field(max_length = 1024)]
    pub password: String,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetRole {
    #[field(max_length = 40)]
    pub role: String,
}
