use super::StoreError;
use mahfel_domain::{User, UserId};

pub trait UserStore: Send + Sync {
    fn get(
        &self,
        id: UserId,
    ) -> impl std::future::Future<Output = Result<Option<User>, StoreError>> + Send;
    fn insert(
        &self,
        user: &User,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
    fn save(&self, user: &User)
    -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
}
