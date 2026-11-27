Library for sandboxing Minecraft instances for modded game launchers

```rust,no_run
use anyhow::{Context, Result};
use modrinth_sandbox::{minecraft::MinecraftCommand, SandboxEnv};

#[tokio::main]
async fn main() -> Result<()> {
	// Handle sandbox setup when this executable is launched as a helper.
	if modrinth_sandbox::helper::run_default()? {
		return Ok(());
	}

	// Create the sandbox.
	let sandbox_env = SandboxEnv::new()
		.await
		.context("creating sandbox environment")?;
	let mut command = MinecraftCommand::default();
	command.jre_path = "/path/to/the/jre".into();
	command.classpath = vec!["/path/to/minecraft.jar".into()];
	command.instance_path = "/path/to/instance".into();
	command.persistent_dir = Some("/path/to/instance-data".into());
	// etc.

	// Spawn Minecraft.
	let command = modrinth_sandbox::minecraft::setup_minecraft_command(command)
		.await
		.context("creating Minecraft sandbox command")?;
	let mut minecraft_process = sandbox_env
		.spawn(command)
		.await
		.context("running Minecraft")?;
	// Minecraft is now running in a sandbox.

	// Wait for the user to close the game.
	minecraft_process.wait().await?;
	Ok(())
}
```

The goal of this crate is to make it stupid simple to run a Minecraft instance inside a sandbox _the right way_. Sandboxing isolates the game process from the user's real machine, so that mods and other game code is unable to access sensitive user resources, like browser cookies or login tokens; this prevents malicious mods from reading such information, or performing other sensitive actions.

Sandboxing aims to be minimally invasive to the game, so most mods should continue working fine. This means we generally do not restrict access to the network, microphone, controller, or other hardware. Mods which read files outside of the instance directory may require changes to work properly.

# Architecture

We use OS-specific primitives to perform sandboxing, that is:
- Windows [AppContainers]
- macOS [Seatbelt]
- Linux
	- In Flatpak: the [`org.freedesktop.portal.Flatpak`] portal
	- Outside of Flatpak: [Bubblewrap]

Each platform has its own specific capabilities, but we abstract over that using the `SandboxCommand`, which is a platform-independent way to configure:
- What command you want to run in a sandbox
- What capabilities you allow inside the sandbox
- What files are exposed inside the sandbox

The general flow of spawning a sandboxed process is:

- Call `helper::run_default`

	This handles sandbox setup when the application is relaunched as a helper process. If it returns `true`, exit without continuing normal application startup.

- Create a `SandboxEnv`

	This ensures that we have the right dependencies to actually perform sandboxing. On most platforms this should always be successful, but on Linux we may be missing `bwrap` - this is our chance to inform the user that they're missing dependencies.

- Create a `SandboxCommand`

	This sets up the sandboxing configuration and describes what we actually want to run.

- Run `SandboxEnv::spawn`

	This starts the sandboxed process, and gives us back a `SandboxChild`, which we can use to wait for process termination, or kill the process, or anything else we need to do.

[AppContainers]: https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation
[Seatbelt]: https://theapplewiki.com/wiki/Dev:Seatbelt
[`org.freedesktop.portal.Flatpak`]: https://docs.flatpak.org/en/latest/libflatpak-api-reference.html#gdbus-org.freedesktop.portal.Flatpak
[Bubblewrap]: https://github.com/containers/bubblewrap
