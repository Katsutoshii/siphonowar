use bevy::ecs::query::QueryEntityError;

pub enum Error {
    Default,
    QueryEntityError(QueryEntityError),
}
impl From<QueryEntityError> for Error {
    fn from(value: QueryEntityError) -> Self {
        Self::QueryEntityError(value)
    }
}
