use bevy::prelude::*;
use bevy_assets_loader::AssetLoaderPlugin;

#[test]
fn no_assets() {
    App::build()
        .insert_resource(0 as i32)
        .add_state(MyStates::Load)
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_update(MyStates::Load).with_system(print.system()))
        .add_plugin(AssetLoaderPlugin::new(MyStates::Load, MyStates::Next))
        .set_runner(|mut app| {
            app.schedule.run(&mut app.world);
            app.schedule.run(&mut app.world);
        })
        .run();
}

fn print(mut frame_count: ResMut<i32>) {
    panic!("The asset loader should have changed the state on enter of the loading state");
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Next,
}
