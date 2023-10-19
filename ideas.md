API could be split roughly following the add_systems/configure_sets APIs from Bevy

e.g.

app.configure_loading_state(MySate::Loading, add_collection::<MyCollection>().init_resource::<SomeResource>())

This cuts down on boilerplate and is more "Bevy like"