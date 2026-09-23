Library for sandboxing Minecraft instances for modded game launchers

# Examples

```rs
use eyre::{Context, Result};
use modrinth_sandbox::{MinecraftCommand, SandboxOutput};

#[tokio::main]
async fn main() -> Result<()> {
	let sandbox_env = modrinth_sandbox::create_env()
		.await
		.wrap_err("creating sandbox environment")?;
	let command = MinecraftCommand {
		jre_path: "/path/to/the/jre".into(),
		classpath: vec!["/path/to/minecraft.jar".into()],
		instance_path: "/path/to/instance".into(),
		// etc.
	};
	let command = modrinth_sandbox::create_minecraft_command(command)
		.wrap_err("creating Minecraft sandbox command")?;
	let mut minecraft_process = sandbox_env
		.spawn(command)
		.await
		.wrap_err("running Minecraft")?;
	// you now have Minecraft running in a sandbox!

  // wait for the user to close the game
	minecraft_process.wait().await?;
	Ok(())
}
```
