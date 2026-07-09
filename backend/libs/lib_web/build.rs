use std::{env, path::PathBuf, process::Command};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // backend/libs/lib_http is three levels deep from the workspace root,
    // which holds `frontend/`.
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("expected workspace root three levels above CARGO_MANIFEST_DIR");

    let frontend_src = workspace_root.join("frontend/src");
    let frontend_dir = workspace_root.join("frontend");

    // Re-run this script whenever anything inside frontend/src changes.
    println!("cargo:rerun-if-changed={}", frontend_src.display());

    let status = Command::new("pnpm")
        .args(["run", "build"])
        .current_dir(&frontend_dir)
        .status()
        .expect("failed to spawn `npm run build`");

    assert!(status.success(), "`npm run build` exited with {status}");
}
