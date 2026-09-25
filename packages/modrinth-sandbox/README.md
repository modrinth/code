Library for sandboxing Minecraft instances for modded game launchers

```rs
use eyre::{Context, Result};
use modrinth_sandbox::{minecraft::MinecraftCommand, SandboxOutput};

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
	let command = modrinth_sandbox::minecraft::create_command(command)
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

The goal of this crate is to make it stupid simple to run a Minecraft instance inside a sandbox _the right way_. Sandboxing isolates the game process from the user's real machine, so that mods and other game code is unable to access sensitive user resources, like browser cookies or login tokens; this prevents malicious mods from reading such information, or performing other sensitive actions.

Sandboxing aims to be minimally invasive to the game, so most mods should continue working fine. This means we generally do not restrict access to the network, microphone, controller, or other hardware. Mods which read files outside of the instance directory may require changes to work properly.

# Architecture

We use OS-specific primitives to perform sandboxing, that is:
- Windows [AppContainers]
- MacOS [Seatbelt] and `sandbox-exec`
- Linux
	- In Flatpak: [`flatpak-spawn`]
	- Outside of Flatpak: [Bubblewrap]

Each platform has its own specific capabilities, but we abstract over that using the `SandboxCommand`, which is a platform-independent way to configure:
- What command you want to run in a sandbox
- What capabilities you allow inside the sandbox
- What files are exposed inside the sandbox

The general flow of spawning a sandboxed process is:

- Create a `SandboxEnv`

	This ensures that we have the right dependencies to actually perform sandboxing. On most platforms this should always be successful, but on Linux we may be missing `bwrap` - this is our chance to inform the user that they're missing dependencies.

- Create a `SandboxCommand`

	This sets up the sandboxing configuration and describes what we actually want to run.

- Run `SandboxEnv::spawn`

	This starts the sandboxed process, and gives us back a `SandboxChild`, which we can use to wait for process termination, or kill the process, or anything else we need to do.

[AppContainers]: https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation
[Seatbelt]: https://theapplewiki.com/wiki/Dev:Seatbelt
[`flatpak-spawn`]: https://www.man7.org/linux/man-pages/man1/flatpak-spawn.1.html
[Bubblewrap]: https://github.com/containers/bubblewrap
