use std::{any::type_name, marker::PhantomData};

use crate::loading_state::config::ScheduleConfig;
use crate::prelude::AssetCollection;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::prelude::{BevyError, IntoScheduleConfigs};
use bevy::{
    asset::UntypedHandle,
    prelude::Component,
};
use bevy::ecs::system::SystemId;

#[derive(Component)]
pub struct AssetCollectionNode {
    pub name: String,
    pub add: SystemId,
    pub load: SystemId<(), Result<Vec<UntypedHandle>, BevyError>>,
}

impl AssetCollectionNode {
    fn new(config: AssetCollectionNodeConfig, world: &mut World) -> Self {
        let add = world.register_system(config.add);
        let load = world.register_system(config.load);

        AssetCollectionNode { name: config.name, add, load }
    }
}

pub struct AssetCollectionNodeConfig {
    pub name: String,
    pub add: ScheduleConfig,
    pub load: ScheduleConfig<Vec<UntypedHandle>>
}

impl AssetCollectionNodeConfig {
    pub fn new<A: AssetCollection>() -> Self {
        let add = A::add.into_configs();
        let load = A::load.into_configs();
        let name = type_name::<A>().to_owned();

        AssetCollectionNodeConfig { add, load, name }
    }
}

#[derive(Component)]
pub struct LoadingHandles {
    pub handles: Vec<UntypedHandle>,
}

#[derive(Component)]
pub struct LoadingStateMarker<S> {
    _marker: PhantomData<S>,
}
