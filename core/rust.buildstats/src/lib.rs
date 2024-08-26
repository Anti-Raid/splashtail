// Various statistics
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_SHA: &str = env!("__BUILDSTATS__GIT_COMMIT_HASH");
pub const GIT_REPO: &str = env!("__BUILDSTATS__GIT_REPO");
pub const GIT_COMMIT_MSG: &str = env!("__BUILDSTATS__GIT_COMMIT_MESSAGE");
pub const BUILD_CPU: &str = env!("__BUILDSTATS__CPU_MODEL");
pub const CARGO_PROFILE: &str = env!("__BUILDSTATS__CARGO_PROFILE");
pub const RUSTC_VERSION: &str = env!("__BUILDSTATS__RUSTC_VERSION");
