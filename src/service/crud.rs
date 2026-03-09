use crate::domain::Identifiable;

pub trait Create<D, C> {
    type Error;

    async fn create(&self, payload: C) -> Result<D, Self::Error>;
}

pub trait Read<D>
where
    D: Identifiable + Send + Sync,
{
    type Error;

    async fn read(&self, id: <D as Identifiable>::Id) -> Result<D, Self::Error>;
    async fn read_page(&self, page: u64, amount: u64) -> Result<Vec<D>, Self::Error>;
    async fn read_all(&self) -> Result<Vec<D>, Self::Error>;
}

pub trait Update<D, U>
where
    D: Identifiable + Send + Sync,
    U: Identifiable
{
    type Error;

    async fn update(&self, payload: U) -> Result<D, Self::Error>;
}

pub trait Delete<D>
where
    D: Identifiable + Send + Sync,
{
    type Error;

    async fn delete(&self, id: <D as Identifiable>::Id) -> Result<(), Self::Error>;
}