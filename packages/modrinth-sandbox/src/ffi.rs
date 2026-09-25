// todo

#[unsafe(no_mangle)]
pub extern "C" fn modrinth_sandbox_create_env() {
    let rt = tokio::runtime::Runtime::new().

    crate::create_env().await
}

pub struct ModrinthSandboxEnv {
    env: crate::SandboxEnv,
    rt: tokio::runtime::Runtime,
}

#[unsafe(no_mangle)]
pub extern "C" fn modrinth_sandbox_env_free(env: *mut ModrinthSandboxEnv) {
    if !env.is_null() {
        unsafe {
            drop(Box::from_raw(env));
        }
    }
}
