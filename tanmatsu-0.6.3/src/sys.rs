#[cfg(not(target_os = "redox"))]
mod other;
#[cfg(target_os = "redox")]
mod redox;

// #[cfg(target_os = "windows")]
// mod other;
// #[cfg(not(target_os = "windows"))]
// mod redox;
