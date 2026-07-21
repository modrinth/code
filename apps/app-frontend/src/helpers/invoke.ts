import { invoke as tauriInvoke } from '@tauri-apps/api/core'

export type OperationCause =
	| 'app/startup'
	| 'navigation/home'
	| 'navigation/library'
	| 'navigation/browse'
	| 'navigation/project'
	| 'navigation/instance/overview'
	| 'navigation/instance/content'
	| 'navigation/instance/logs'
	| 'navigation/servers'
	| 'navigation/server/manage'
	| 'navigation/server/content'
	| 'instance/refresh/user'
	| 'instance/refresh/filesystem_watch'
	| 'instance/update/all'
	| 'instance/update/single'
	| 'instance/install'
	| 'cache/revalidate'
	| 'auth/session_refresh'
	| 'background/friends'
	| 'minecraft/launch'
	| 'app/update_check'
	| 'unattributed'

function navigationCause(pathname: string): OperationCause {
	if (pathname === '/hosting/manage' || pathname === '/hosting/manage/') {
		return 'navigation/servers'
	}
	if (pathname.startsWith('/hosting/manage/')) {
		return pathname.split('/').includes('content')
			? 'navigation/server/content'
			: 'navigation/server/manage'
	}
	if (pathname.startsWith('/browse/')) return 'navigation/browse'
	if (pathname.startsWith('/project/')) return 'navigation/project'
	if (pathname.startsWith('/library')) return 'navigation/library'
	if (pathname.startsWith('/instance/')) {
		return pathname.split('/').includes('logs')
			? 'navigation/instance/logs'
			: 'navigation/instance/content'
	}

	return 'navigation/home'
}

function commandCause(command: string): OperationCause {
	if (command === 'plugin:instance|instance_update_all') return 'instance/update/all'
	if (
		command === 'plugin:instance|instance_update_project' ||
		command === 'plugin:instance|instance_switch_project_version_with_dependencies' ||
		command === 'plugin:instance|instance_update_managed_modrinth_version'
	) {
		return 'instance/update/single'
	}
	if (
		command.startsWith('plugin:install|install_') ||
		command === 'plugin:jre|jre_auto_install_java' ||
		command === 'plugin:instance|instance_add_project_from_version' ||
		command === 'plugin:instance|instance_install_project_with_dependencies' ||
		command === 'plugin:instance|instance_repair_managed_modrinth'
	) {
		return 'instance/install'
	}
	if (
		command === 'plugin:instance|instance_run' ||
		command === 'plugin:worlds|start_join_singleplayer_world' ||
		command === 'plugin:worlds|start_join_server'
	) {
		return 'minecraft/launch'
	}
	if (command === 'plugin:mr-auth|get') return 'auth/session_refresh'

	return navigationCause(window.location.pathname)
}

export function invoke<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
	return tauriInvoke<T>(command, {
		...args,
		invocationContext: { cause: commandCause(command) },
	})
}
