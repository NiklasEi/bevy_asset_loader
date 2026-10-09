use crate::asset_collection::AssetCollection;
use bevy_ecs::error::Result;
use bevy_ecs::template::{Template, TemplateContext, template};
use core::marker::PhantomData;

/// A reference to a single field of an [`AssetCollection`].
pub struct CollectionField<Collection, Field> {
    getter: fn(&Collection) -> &Field,
}

impl<Collection, Field> CollectionField<Collection, Field> {
    /// Create a field reference from a getter.
    pub const fn new(getter: fn(&Collection) -> &Field) -> Self {
        Self { getter }
    }

    /// Read this field from the given collection.
    pub fn get<'a>(&self, collection: &'a Collection) -> &'a Field {
        (self.getter)(collection)
    }
}

impl<Collection, Field> Clone for CollectionField<Collection, Field> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Collection, Field> Copy for CollectionField<Collection, Field> {}

/// The [`Template`] returned by [`FromCollection::from_collection`].
pub struct CollectionFieldTemplate<Collection, Field, Output> {
    field: CollectionField<Collection, Field>,
    marker: PhantomData<fn() -> Output>,
}

impl<Collection, Field, Output> CollectionFieldTemplate<Collection, Field, Output> {
    /// Configure the component that was built from the collection field.
    pub fn map<Mapped, Func>(self, func: Func) -> MappedTemplate<Self, Func, Mapped>
    where
        Func: Fn(Output) -> Mapped + Clone,
    {
        MappedTemplate {
            inner: self,
            func,
            marker: PhantomData,
        }
    }
}

/// The [`Template`] returned by [`CollectionFieldTemplate::map`]. It builds the inner template and
/// passes the result through a function.
pub struct MappedTemplate<Inner, Func, Output> {
    inner: Inner,
    func: Func,
    marker: PhantomData<fn() -> Output>,
}

impl<Inner, Func, Output> Template for MappedTemplate<Inner, Func, Output>
where
    Inner: Template,
    Func: Fn(Inner::Output) -> Output + Clone,
{
    type Output = Output;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Output> {
        Ok((self.func)(self.inner.build_template(context)?))
    }

    fn clone_template(&self) -> Self {
        Self {
            inner: self.inner.clone_template(),
            func: self.func.clone(),
            marker: PhantomData,
        }
    }
}

impl<Collection, Field, Output> Template for CollectionFieldTemplate<Collection, Field, Output>
where
    Collection: AssetCollection,
    Field: Clone,
    Output: From<Field>,
{
    type Output = Output;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Output> {
        Ok(Output::from(
            self.field.get(context.resource::<Collection>()).clone(),
        ))
    }

    fn clone_template(&self) -> Self {
        Self {
            field: self.field,
            marker: PhantomData,
        }
    }
}

/// Build a component from a field of an [`AssetCollection`] inside the `bsn!` macro.
///
/// This is implemented for every [`Template`]. The component is created from the
/// collection's field through [`From`], so this works for all components that can
/// be built from a handle, e.g. `Sprite`, `Mesh3d`, `MeshMaterial3d`, `AudioPlayer` or `TextFont`.
pub trait FromCollection: Template + Sized {
    /// Read `field` from [`Collection`] while the scene is spawned and turn it
    /// into the Template's output component.
    fn from_collection<Collection, Field>(
        field: CollectionField<Collection, Field>,
    ) -> CollectionFieldTemplate<Collection, Field, Self::Output>
    where
        Collection: AssetCollection,
        Field: Clone,
        Self::Output: From<Field>,
    {
        CollectionFieldTemplate {
            field,
            marker: PhantomData,
        }
    }
}

impl<T: Template> FromCollection for T {}

/// Build a value from an [`AssetCollection`] while a scene is being spawned.
pub fn from_collection<Collection, Output, Func>(
    func: Func,
) -> impl Template<Output = Output> + Send + Sync + 'static
where
    Collection: AssetCollection,
    Func: Fn(&Collection) -> Output + Clone + Send + Sync + 'static,
    Output: 'static,
{
    template(move |context: &mut TemplateContext| -> Result<Output> {
        Ok(func(context.resource::<Collection>()))
    })
}
