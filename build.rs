// release-2026-11-14: the web interface embedded with include_str! comes from `web/`, or from the minified copy
// `scripts/minify-web.sh` makes when STELLA_WEB_DIR names it (the release build), so shipped JavaScript carries no comments.
fn main() {
    println!("cargo:rerun-if-env-changed=STELLA_WEB_DIR");
    let dir = std::env::var("STELLA_WEB_DIR")
        .unwrap_or_else(|_| format!("{}/web", std::env::var("CARGO_MANIFEST_DIR").unwrap()));
    println!("cargo:rustc-env=STELLA_WEB={dir}");
}
