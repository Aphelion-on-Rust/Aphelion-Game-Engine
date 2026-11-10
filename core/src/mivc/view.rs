/// Marks what a view is allowed to receive: shared references (`&T`) and tuples of them.
/// `&mut T` is NOT marked, so a view can never be handed something it could change.
pub trait ReadOnly {}

impl<T: ?Sized> ReadOnly for &T {}
impl<A: ReadOnly, B: ReadOnly> ReadOnly for (A, B) {}
impl<A: ReadOnly, B: ReadOnly, C: ReadOnly> ReadOnly for (A, B, C) {}
impl<A: ReadOnly, B: ReadOnly, C: ReadOnly, D: ReadOnly> ReadOnly for (A, B, C, D) {}

/// A view reads models and shows them. It never changes data.
///
/// `Models` is the bundle of models the view reads, written as references:
///   type Models<'a> = (&'a SceneModel, &'a HierarchyUiModel);
///
/// A view can keep its own screen state (the rows it built, scroll position),
/// that is why `render` takes `&mut self`. The models stay read-only.
pub trait View {
    type Models<'a>: ReadOnly;

    fn render(&mut self, models: Self::Models<'_>);
}
