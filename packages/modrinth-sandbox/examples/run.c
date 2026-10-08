/*
 * Arguments and paths must be UTF-8. Unlike run.rs, output is inherited
 * directly instead of being forwarded with a "child:" prefix.
 * Windows helper startup is not yet exposed through the C API.
 */
#include "modrinth_sandbox.h"

#include <inttypes.h>
#include <stdio.h>
#include <string.h>

static ModrinthSandboxString string_view(const char *value)
{
	return (ModrinthSandboxString){value, strlen(value)};
}

static void report_error(const char *operation)
{
	const uint8_t *message = NULL;
	size_t length = 0;
	modrinth_sandbox_get_last_error(&message, &length);
	fprintf(stderr, "%s: ", operation);
	if (length != 0) {
		fwrite(message, 1, length, stderr);
	}
	fputc('\n', stderr);
}

int main(int argc, char **argv)
{
#ifdef _WIN32
	(void)argc;
	(void)argv;
	fprintf(stderr, "This example requires a Windows helper startup FFI binding.\n");
	return EXIT_FAILURE;
#endif

	int result = EXIT_FAILURE;
	ModrinthSandboxEnv *env = NULL;
	ModrinthSandboxCommand *command = NULL;
	ModrinthSandboxChild *child = NULL;
	ModrinthSandboxExitStatus status = {0};
	ModrinthSandboxCommandView view = {0};
	ModrinthSandboxString *read_paths = calloc((size_t)argc, sizeof(*read_paths));
	ModrinthSandboxString *write_paths = calloc((size_t)argc, sizeof(*write_paths));
	ModrinthSandboxString *args = calloc((size_t)argc, sizeof(*args));
	if (!read_paths || !write_paths || !args) {
		fprintf(stderr, "Allocating command arguments failed.\n");
		goto cleanup;
	}

	view.read_only_paths.ptr = read_paths;
	view.read_write_paths.ptr = write_paths;
	view.args.ptr = args;
	int index = 1;
	for (; index < argc; ++index) {
		const char *option = argv[index];
		if (strcmp(option, "--") == 0) {
			++index;
			break;
		}
		if (option[0] != '-') {
			break;
		}
		if (strcmp(option, "--read") != 0 && strcmp(option, "--write") != 0 && strcmp(option, "--cwd") != 0) {
			goto usage;
		}
		if (++index == argc) {
			goto usage;
		}
		ModrinthSandboxString path = string_view(argv[index]);
		if (strcmp(option, "--read") == 0) {
			read_paths[view.read_only_paths.len++] = path;
		} else if (strcmp(option, "--write") == 0) {
			write_paths[view.read_write_paths.len++] = path;
		} else {
			view.working_directory = (ModrinthSandboxStringOption){path.ptr, path.len};
		}
	}
	if (index == argc) {
		goto usage;
	}
	view.executable = string_view(argv[index++]);
	for (; index < argc; ++index) {
		args[view.args.len++] = string_view(argv[index]);
	}
	view.allow_network = true;
	view.die_with_parent = true;
	view.child_stdin = MODRINTH_SANDBOX_STDIO_NULL;
	view.child_stdout = MODRINTH_SANDBOX_STDIO_INHERIT;
	view.child_stderr = MODRINTH_SANDBOX_STDIO_INHERIT;
	view.app_container_name = string_view("ModrinthMinecraftSandbox");
	view.app_container_description = string_view("Sandbox for Minecraft instances created by modrinth-sandbox");

	if (!modrinth_sandbox_create_env(&env)) {
		report_error("Creating sandbox environment");
		goto cleanup;
	}
	if (!modrinth_sandbox_prepare_command(view, &command)) {
		report_error("Preparing sandbox command");
		goto cleanup;
	}
	/* Spawn consumes the command and clears its handle, even on failure. */
	if (!modrinth_sandbox_spawn(env, &command, &child)) {
		report_error("Spawning sandboxed process");
		goto cleanup;
	}
	if (!modrinth_sandbox_child_wait(child, &status)) {
		report_error("Waiting for sandboxed process");
		modrinth_sandbox_child_kill(child);
		goto cleanup;
	}
	if (!status.success) {
		if (status.has_code) {
			fprintf(stderr, "Sandboxed process exited with code %" PRId32 ".\n", status.code);
		} else {
			fprintf(stderr, "Sandboxed process terminated without an exit code.\n");
		}
		goto cleanup;
	}
	result = EXIT_SUCCESS;
	goto cleanup;

usage:
	fprintf(stderr, "Usage: %s [--read PATH] [--write PATH] [--cwd PATH] [--] EXECUTABLE [ARGS...]\n", argv[0]);
cleanup:
	modrinth_sandbox_child_free(child);
	modrinth_sandbox_command_free(command);
	modrinth_sandbox_env_free(env);
	free(args);
	free(write_paths);
	free(read_paths);
	return result;
}
