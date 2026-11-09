import type { Labrinth } from '@modrinth/api-client'
import { type AbstractPopupNotificationManager, createContext } from '@modrinth/ui'
import { type Ref, ref } from 'vue'
import type { Router } from 'vue-router'

import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { get_project, get_project_v3, get_version } from '@/helpers/cache.js'
import {
	install_create_instance,
	install_create_modpack_instance,
	install_existing_instance,
	installJobInstanceId,
	wait_for_install_job,
} from '@/helpers/install'
import { edit, get, list, run } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import {
	ensureManagedServerWorldExists,
	getServerAddress,
	normalizeServerAddress,
} from '@/helpers/worlds'
import { start_join_server } from '@/helpers/worlds.ts'
import type { AppEvents } from '@/providers/app-events'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
interface ModalRef<TShow extends (...args: any[]) => void = () => void> {
	show: TShow
	hide: () => void
}

export interface PlayServerProjectOptions {
	/** Join the server after launching. When false, the game launches to the main menu. Defaults to true. */
	join?: boolean
	/** Analytics source for the `InstanceStart` event. Defaults to `ServerProject`. */
	source?: string
}

/** What a "Launch game" action does: join `serverAddress`, or launch to the main menu when null. */
interface LaunchTarget {
	serverAddress: string | null
	source?: string
}

function getServerProjectId(instance: GameInstance) {
	const link = instance.link
	if (link?.type !== 'server_project' && link?.type !== 'server_project_modpack') return undefined
	return link.project_id ?? link.server_project_id ?? undefined
}

export interface ServerInstallContext {
	installingServerProjects: Ref<string[]>
	startInstallingServer: (projectId: string) => void
	stopInstallingServer: (projectId: string) => void
	isServerInstalling: (projectId: string) => boolean
	installServerProject: (serverProjectId: string) => Promise<void>
	playServerProject: (projectId: string, options?: PlayServerProjectOptions) => Promise<void>
	joinInstanceServer: (instance: GameInstance, address: string, source?: string) => Promise<void>
	launchInstance: (instance: GameInstance, source?: string) => Promise<void>
	setInstallToPlayModal: (
		ref: ModalRef<
			(
				project: Labrinth.Projects.v3.Project,
				modpackVersionId: string | null,
				callback?: () => void,
			) => Promise<void>
		>,
	) => void
	setUpdateToPlayModal: (
		ref: ModalRef<
			(instance: GameInstance, activeVersionId: string | null, callback?: () => void) => void
		>,
	) => void
	setAddServerToInstanceModal: (
		ref: ModalRef<(serverName: string, serverAddress: string) => void>,
	) => void
	showAddServerToInstanceModal: (serverName: string, serverAddress: string) => void
}

let _serverInstallSingleton: ServerInstallContext | null = null

const [_rawInjectServerInstall, provideServerInstall] = createContext<ServerInstallContext>(
	'root',
	'serverInstall',
)

export { provideServerInstall }

export function injectServerInstall(): ServerInstallContext {
	try {
		return _rawInjectServerInstall()
	} catch {
		if (_serverInstallSingleton) return _serverInstallSingleton
		throw new Error('ServerInstall context not available')
	}
}

export function createServerInstall(opts: {
	router: Router
	handleError: (err: unknown) => void
	popupNotificationManager: AbstractPopupNotificationManager
	appEvents: AppEvents
}): ServerInstallContext {
	const installingServerProjects = ref<string[]>([])

	let installToPlayModalRef: ModalRef<
		(
			project: Labrinth.Projects.v3.Project,
			modpackVersionId: string | null,
			callback?: () => void,
		) => Promise<void>
	> | null = null
	let updateToPlayModalRef: ModalRef<
		(instance: GameInstance, activeVersionId: string | null, callback?: () => void) => void
	> | null = null
	let addServerToInstanceModalRef: ModalRef<
		(serverName: string, serverAddress: string) => void
	> | null = null

	function startInstallingServer(projectId: string) {
		if (!installingServerProjects.value.includes(projectId)) {
			installingServerProjects.value.push(projectId)
		}
	}

	function stopInstallingServer(projectId: string) {
		installingServerProjects.value = installingServerProjects.value.filter((id) => id !== projectId)
	}

	function isServerInstalling(projectId: string) {
		return installingServerProjects.value.includes(projectId)
	}

	/** Launches an instance, joining `serverAddress` when given, and tracks the start when `source` is set. */
	async function launchGame(instance: GameInstance, serverAddress: string | null, source?: string) {
		if (serverAddress) await start_join_server(instance.id, serverAddress)
		else await run(instance.id)
		if (source) {
			trackEvent('InstanceStart', {
				loader: instance.loader,
				game_version: instance.game_version,
				source,
			})
		}
	}

	function launchGameButton(instance: GameInstance, launch: LaunchTarget | null) {
		if (!launch) return []
		return [
			{
				label: 'Launch game',
				action: async () => {
					try {
						await launchGame(instance, launch.serverAddress, launch.source)
					} catch (err) {
						handleSevereError(err, { instanceId: instance.id })
					}
				},
				color: 'brand' as const,
			},
		]
	}

	async function findInstalledInstance(projectId: string) {
		const packs = await list()
		return packs.find((pack) => pack.link?.project_id === projectId) ?? null
	}

	async function createVanillaInstance(
		project: Labrinth.Projects.v2.Project,
		gameVersion: string,
		serverAddress: string | null,
	) {
		const job = await install_create_instance({
			name: project.title,
			gameVersion,
			loader: 'vanilla',
			loaderVersion: null,
			iconPath: project.raw_icon_url ?? null,
			link: {
				type: 'server_project',
				project_id: project.id,
			},
		})
		const instanceId = installJobInstanceId(job)
		if (!instanceId) return null

		await wait_for_install_job(opts.appEvents, job.job_id)
		await ensureManagedServerWorldExists(instanceId, project.title, serverAddress)

		return instanceId
	}

	async function updateVanillaGameVersion(instance: GameInstance, targetGameVersion: string) {
		if (instance.game_version === targetGameVersion) return

		await edit(instance.id, { game_version: targetGameVersion })
		const job = await install_existing_instance(instance.id, false)
		await wait_for_install_job(opts.appEvents, job.job_id)
	}

	function showModpackInstallSuccess(project: GameInstance, launch: LaunchTarget | null) {
		opts.popupNotificationManager.addPopupNotification({
			contentType: 'standard',
			title: 'Install complete',
			text: `${project.name} is installed and ready to play.`,
			type: 'success',
			buttons: [
				...launchGameButton(project, launch),
				{
					label: 'Instance',
					action: () => opts.router.push(`/instance/${encodeURIComponent(project.id)}`),
				},
			],
			autoCloseMs: null,
		})
	}

	function showUpdateSuccess(instance: GameInstance, launch: LaunchTarget | null) {
		opts.popupNotificationManager.addPopupNotification({
			contentType: 'standard',
			title: 'Update complete',
			text: `${instance.name} has been updated and is ready to play.`,
			type: 'success',
			buttons: [
				...launchGameButton(instance, launch),
				{
					label: 'Instance',
					action: () => opts.router.push(`/instance/${encodeURIComponent(instance.id)}`),
				},
			],
			autoCloseMs: null,
		})
	}

	/**
	 * Server projects that use modpack content have link.project_id as
	 * the server project id and link.version_id as the modpack content version id.
	 * The modpack content version can be of the same server project, or from a different project.
	 */
	async function installServerProject(serverProjectId: string) {
		const [project, projectV3] = await Promise.all([
			get_project(serverProjectId, 'bypass'),
			get_project_v3(serverProjectId, 'bypass'),
		])

		const serverAddress = getServerAddress(projectV3?.minecraft_java_server)

		const content = projectV3?.minecraft_java_server?.content
		if (!content || content.kind !== 'modpack') return

		const contentVersionId = content.version_id
		const contentVersion = await get_version(contentVersionId, 'bypass')
		const contentProjectId = contentVersion.project_id

		const createJob = await install_create_modpack_instance(
			{
				type: 'fromVersionId',
				project_id: contentProjectId,
				version_id: contentVersionId,
				title: project.title,
				icon_url: project.raw_icon_url,
			},
			{
				name: project.title,
				link: {
					type: 'server_project_modpack',
					server_project_id: serverProjectId,
					content_project_id: contentProjectId,
					content_version_id: contentVersionId,
					project_id: serverProjectId,
					version_id: contentVersionId,
				},
			},
		)
		const instanceId = installJobInstanceId(createJob)
		if (!instanceId) return

		await wait_for_install_job(opts.appEvents, createJob.job_id)
		await ensureManagedServerWorldExists(instanceId, project.title, serverAddress)
	}

	/**
	 * Handles logic when clicking "Play" on a server project. This includes:
	 * - Checking if need to install modpack content. If so, opens install to play modal
	 * - Checking if need to update modpack content. If so, open update to play modal
	 * - Checking if need to create instance for vanilla server. If so, creates instance.
	 * - Adding server to worlds list if not already there
	 * - Joining server, or launching to the main menu when `options.join` is false
	 */
	async function playServerProject(projectId: string, options: PlayServerProjectOptions = {}) {
		const { join = true, source = 'ServerProject' } = options
		const [project, projectV3] = await Promise.all([
			get_project(projectId, 'bypass'),
			get_project_v3(projectId, 'bypass'),
		])

		if (projectV3?.minecraft_server == null) {
			console.warn('playServerProject failed: project is not a server project')
			return
		}

		const content = projectV3?.minecraft_java_server?.content
		const serverAddress = getServerAddress(projectV3?.minecraft_java_server)
		const isVanilla = content?.kind === 'vanilla'
		const isModpack = content?.kind === 'modpack'
		const modpackVersionId = content?.version_id ?? null
		const recommendedGameVersion = content?.recommended_game_version
		const launch: LaunchTarget | null = join
			? serverAddress
				? { serverAddress, source }
				: null
			: { serverAddress: null, source }

		let instance = await findInstalledInstance(project.id)

		if (isVanilla && !instance) {
			if (installingServerProjects.value.includes(projectId)) return
			startInstallingServer(projectId)
			try {
				const instanceId = await createVanillaInstance(
					project,
					recommendedGameVersion,
					serverAddress,
				)
				if (instanceId) {
					instance = await get(instanceId)
					if (instance) showModpackInstallSuccess(instance, launch)
				}
			} finally {
				stopInstallingServer(projectId)
			}
			return
		}
		if (isModpack && !instance) {
			await installToPlayModalRef?.show(projectV3, modpackVersionId, async () => {
				const newInstance = await findInstalledInstance(project.id)
				if (!newInstance) return
				showModpackInstallSuccess(newInstance, launch)
			})
			return
		}

		if (!instance) return

		await ensureManagedServerWorldExists(instance.id, project.title, serverAddress)

		// Update existing instance if needed
		if (isModpack && instance.link?.version_id !== modpackVersionId) {
			updateToPlayModalRef?.show(instance, modpackVersionId, () => {
				showUpdateSuccess(instance, launch)
			})
			return
		}
		if (isVanilla && instance.game_version !== recommendedGameVersion) {
			if (installingServerProjects.value.includes(projectId)) return
			startInstallingServer(projectId)
			try {
				await updateVanillaGameVersion(instance, recommendedGameVersion)
				showUpdateSuccess(instance, launch)
			} finally {
				stopInstallingServer(projectId)
			}
			return
		}

		if (!launch) return
		try {
			await launchGame(instance, launch.serverAddress, launch.source)
		} catch (err) {
			handleSevereError(err, { instanceId: instance.id })
		}
	}

	/**
	 * Joins a server listed in an instance's worlds. When the instance is linked to a server
	 * project and the address is that project's server, this goes through `playServerProject`
	 * so outdated modpack content prompts an update before launching.
	 */
	async function joinInstanceServer(instance: GameInstance, address: string, source?: string) {
		const serverProjectId = getServerProjectId(instance)

		if (serverProjectId) {
			const projectV3 = await get_project_v3(serverProjectId).catch((err) => {
				console.error(`Failed to resolve server project ${serverProjectId}:`, err)
				return null
			})
			const projectAddress = getServerAddress(projectV3?.minecraft_java_server)
			if (
				projectAddress &&
				normalizeServerAddress(projectAddress) === normalizeServerAddress(address)
			) {
				await playServerProject(serverProjectId, { source })
				return
			}
		}

		await launchGame(instance, address, source)
	}

	/**
	 * Launches an instance without joining a server. Instances linked to a server project go
	 * through `playServerProject` so outdated server content prompts an update before launching.
	 */
	async function launchInstance(instance: GameInstance, source?: string) {
		const serverProjectId = getServerProjectId(instance)
		if (serverProjectId) {
			await playServerProject(serverProjectId, { join: false, source })
			return
		}

		await launchGame(instance, null, source)
	}

	const context: ServerInstallContext = {
		installingServerProjects,
		startInstallingServer,
		stopInstallingServer,
		isServerInstalling,
		installServerProject,
		playServerProject,
		joinInstanceServer,
		launchInstance,
		setInstallToPlayModal(ref) {
			installToPlayModalRef = ref
		},
		setUpdateToPlayModal(ref) {
			updateToPlayModalRef = ref
		},
		setAddServerToInstanceModal(ref) {
			addServerToInstanceModalRef = ref
		},
		showAddServerToInstanceModal(serverName: string, serverAddress: string) {
			addServerToInstanceModalRef?.show(serverName, serverAddress)
		},
	}

	_serverInstallSingleton = context
	return context
}
