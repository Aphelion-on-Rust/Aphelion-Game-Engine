use super::model::Model;

/// Proof that the caller is a controller.
///
/// The field is private, so the only place a token can be created is in this file.
/// That is how the engine makes sure only controllers change models.
pub struct WriteToken(());

/// A controller holds the logic. It is the only thing that may change a model.
///
/// Implement it on your controller with an empty body: `impl Controller for MyController {}`
/// then call `self.write(model)` whenever you need to change data.
pub trait Controller {
    fn write<'a, M: Model>(&self, model: &'a mut M) -> &'a mut M::Data {
        model.data_mut(WriteToken(()))
    }
}
