use super::StoreError;
use mahfel_domain::{ClassroomSession, ClassroomSessionId};

pub trait SessionStore: Send + Sync {
    fn get(
        &self,
        id: ClassroomSessionId,
    ) -> impl std::future::Future<Output = Result<Option<ClassroomSession>, StoreError>> + Send;
    fn insert(
        &self,
        session: &ClassroomSession,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
    fn save(
        &self,
        session: &ClassroomSession,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
}
