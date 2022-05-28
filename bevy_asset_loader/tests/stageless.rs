#[cfg(all(
    feature = "stageless",
    not(feature = "2d"),
    not(feature = "3d"),
    not(feature = "progress_tracking")
))]
mod stageless {
    mod can_run_without_next_state;
    mod init_resource;
    mod multiple_asset_collections;
}
