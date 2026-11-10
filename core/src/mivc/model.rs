use super::controller::WriteToken;
use super::interface::Interface;

/// A model only stores data. It has no game logic.
///
/// Reading is open to everyone (`data`).
/// Writing needs a `WriteToken`, and only a Controller can make one.
pub trait Model {
    type Data: Interface;

    fn data(&self) -> &Self::Data;
    fn data_mut(&mut self, token: WriteToken) -> &mut Self::Data;
}

/// The simplest model: it just holds one piece of data.
/// Most models in the engine are a `Store<SomeData>`.
pub struct Store<D: Interface> {
    data: D,
}

impl<D: Interface> Store<D> {
    pub fn new(data: D) -> Self {
        Self { data }
    }
}

impl<D: Interface + Default> Default for Store<D> {
    fn default() -> Self {
        Self::new(D::default())
    }
}

impl<D: Interface> Model for Store<D> {
    type Data = D;

    fn data(&self) -> &D {
        &self.data
    }

    fn data_mut(&mut self, _token: WriteToken) -> &mut D {
        &mut self.data
    }
}
