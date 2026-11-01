//! The app's build (app/: two files) embedded with `embedded-app` — the
//! line every app using plexus-webapp's embedding writes.
fn main() {
    plexus_webapp::build::embed_dir_with_feature("app", "embedded-app");
}
