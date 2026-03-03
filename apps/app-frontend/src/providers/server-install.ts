import type { Labrinth } from '@modrinth/api-client'
import type { AbstractPopupNotificationManager } from '@modrinth/ui'
import { createContext } from '@modrinth/ui'
import { type Ref, ref } from 'vue'
import type { Router } from 'vue-router'

import { get_project, get_project_v3, get_version } from '@/helpers/cache.js'
import { install_to_existing_profile } from '@/helpers/pack.js'
import {
	create,
	edit,
	edit_icon,
	get,
	install as installProfile,
	list,
} from '@/helpers/profile.js'
import {
	add_server_to_profile,
	edit_server_in_profile,
	get_profile_worlds,
	start_join_server,
} from '@/helpers/worlds.ts'
import { handleSevereError } from '@/store/error.js'
import { getServerAddress } from '@/store/install.js'

interface ModalRef<TShow extends (...args: any[]) => void = () => void> {
	show: TShow
	hide: () => void
}

export interface ServerInstallContext {
	installingServerProjects: Ref<string[]>
	startInstallingServer: (projectId: string) => void
	stopInstallingServer: (projectId: string) => void
	isServerInstalling: (projectId: string) => boolean
	installServerProject: (serverProjectId: string) => Promise<void>
	playServerProject: (projectId: string) => Promise<void>
	setInstallToPlayModal: (
		ref: ModalRef<
			(
				project: Labrinth.Projects.v3.Project,
				modpackVersionId: string | null,
				callback?: () => void,
			) => void
		>,
	) => void
	setUpdateToPlayModal: (
		ref: ModalRef<
			(instance: any, activeVersionId: string | null, callback?: () => void) => void
		>,
	) => void
	setAddServerToInstanceModal: (
		ref: ModalRef<(serverName: string, serverAddress: string) => void>,
	) => void
	showAddServerToInstanceModal: (serverName: string, serverAddress: string) => void
}

export const [injectServerInstall, provideServerInstall] = createContext<ServerInstallContext>(
	'root',
	'serverInstall',
)

export function createServerInstall(opts: {
	router: Router
	handleError: (err: unknown) => void
	popupNotificationManager: AbstractPopupNotificationManager
}): ServerInstallContext {
	const installingServerProjects = ref<string[]>([])

	let installToPlayModalRef: ModalRef<
		(
			project: Labrinth.Projects.v3.Project,
			modpackVersionId: string | null,
			callback?: () => void,
		) => void
	> | null = null
	let updateToPlayModalRef: ModalRef<
		(instance: any, activeVersionId: string | null, callback?: () => void) => void
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
		installingServerProjects.value = installingServerProjects.value.filter(
			(id) => id !== projectId,
		)
	}

	function isServerInstalling(projectId: string) {
		return installingServerProjects.value.includes(projectId)
	}

	async function syncServerAsWorld(
		profilePath: string,
		serverName: string,
		serverAddress: string | null,
		serverProjectId: string | null = null,
	) {
		if (!profilePath || !serverAddress) return
		try {
			const worlds = await get_profile_worlds(profilePath)

			if (serverProjectId) {
				const linkedWorld = worlds.find(
					(w: any) => w.type === 'server' && w.linked_project_id === serverProjectId,
				)
				if (linkedWorld) {
					if (linkedWorld.address !== serverAddress || linkedWorld.name !== serverName) {
						await edit_server_in_profile(
							profilePath,
							linkedWorld.index,
							serverName,
							serverAddress,
							linkedWorld.pack_status,
							serverProjectId,
						)
					}
					return
				}
			}

			const existingServer = worlds.find(
				(w: any) => w.type === 'server' && w.address === serverAddress,
			)
			if (existingServer) {
				if (serverProjectId || existingServer.name !== serverName) {
					await edit_server_in_profile(
						profilePath,
						existingServer.index,
						serverName,
						serverAddress,
						existingServer.pack_status,
						serverProjectId ?? undefined,
					)
				}
			} else {
				await add_server_to_profile(
					profilePath,
					serverName,
					serverAddress,
					'prompt',
					serverProjectId ?? undefined,
				)
			}
		} catch (err) {
			console.error('Failed to add server to instance worlds:', err)
		}
	}

	async function joinServer(profilePath: string, serverAddress: string | null) {
		if (!serverAddress) return
		await start_join_server(profilePath, serverAddress)
	}

	async function findInstalledInstance(projectId: string) {
		const packs = await list()
		return packs.find((pack: any) => pack.linked_data?.project_id === projectId) ?? null
	}

	async function createVanillaInstance(
		project: Labrinth.Projects.v2.Project,
		gameVersion: string,
		serverAddress: string | null,
	) {
		const profilePath = await create(
			project.title,
			gameVersion,
			'fabric',
			'latest',
			project.icon_url,
			false,
			{
				project_id: project.id,
				version_id: '',
				locked: true,
			},
		)

		await syncServerAsWorld(profilePath, project.title, serverAddress, project.id)

		return profilePath
	}

	async function updateVanillaGameVersion(instance: any, targetGameVersion: string) {
		if (instance.game_version === targetGameVersion) return

		await edit(instance.path, { game_version: targetGameVersion })
		await installProfile(instance.path, false)
	}

	function showModpackInstallSuccess(project: any, serverAddress: string | null) {
		opts.popupNotificationManager.addPopupNotification({
			title: 'Install complete',
			text: `${project.name} is installed and ready to play.`,
			type: 'success',
			buttons: [
				...(serverAddress
					? [
							{
								label: 'Launch game',
								action: async () => {
									try {
										await joinServer(project.path, serverAddress)
									} catch (err) {
										handleSevereError(err, { profilePath: project.path })
									}
								},
								color: 'brand' as const,
							},
						]
					: []),
				{
					label: 'Instance',
					action: () =>
						opts.router.push(`/instance/${encodeURIComponent(project.path)}`),
				},
			],
			autoCloseMs: null,
		})
	}

	function showUpdateSuccess(instance: any, serverAddress: string | null) {
		opts.popupNotificationManager.addPopupNotification({
			title: 'Update complete',
			text: `${instance.name} has been updated and is ready to play.`,
			type: 'success',
			buttons: [
				...(serverAddress
					? [
							{
								label: 'Launch game',
								action: async () => {
									try {
										if (serverAddress)
											await start_join_server(instance.path, serverAddress)
									} catch (err) {
										handleSevereError(err, { profilePath: instance.path })
									}
								},
								color: 'brand' as const,
							},
						]
					: []),
				{
					label: 'Instance',
					action: () =>
						opts.router.push(`/instance/${encodeURIComponent(instance.path)}`),
				},
			],
			autoCloseMs: null,
		})
	}

	/**
	 * Server projects that use modpack content have linked_data.project_id as
	 * the server project id and linked_data.version_id as the modpack content version id.
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
		const gameVersion = contentVersion.game_versions?.[0] ?? ''

		const profilePath = await create(
			project.title,
			gameVersion,
			'vanilla',
			null,
			project.icon_url,
			true,
			{
				project_id: serverProjectId,
				version_id: contentVersionId,
				locked: true,
			},
		)

		// Save the icon path before pack install overwrites it
		const profileBeforeInstall = await get(profilePath)
		const originalIconPath = profileBeforeInstall?.icon_path ?? null

		await install_to_existing_profile(
			contentProjectId,
			contentVersionId,
			project.title,
			profilePath,
		)

		// Pack install overwrites name, icon, and linked_data with the content project's values.
		// Restore them to point to the server project.
		await edit(profilePath, {
			name: project.title,
			linked_data: {
				project_id: serverProjectId,
				version_id: contentVersionId,
				locked: true,
			},
		})
		await edit_icon(profilePath, originalIconPath)

		await syncServerAsWorld(profilePath, project.title, serverAddress, serverProjectId)
	}

	/**
	 * Handles logic when clicking "Play" on a server project. This includes:
	 * - Checking if need to install modpack content. If so, opens install to play modal
	 * - Checking if need to update modpack content. If so, open update to play modal
	 * - Checking if need to create instance for vanilla server. If so, creates instance.
	 * - Adding server to worlds list if not already there
	 * - Joining server
	 */
	async function playServerProject(projectId: string) {
		const [project, projectV3] = await Promise.all([
			get_project(projectId, 'bypass'),
			get_project_v3(projectId, 'bypass'),
		])

		if (projectV3?.minecraft_server == null) {
			console.warn('playServerProject failed: project is not a server project')
		}

		const content = projectV3?.minecraft_java_server?.content
		const serverAddress = getServerAddress(projectV3?.minecraft_java_server)
		const isVanilla = content?.kind === 'vanilla'
		const isModpack = content?.kind === 'modpack'
		const modpackVersionId = content?.version_id ?? null
		const recommendedGameVersion = content?.recommended_game_version

		let instance = await findInstalledInstance(project.id)

		if (isVanilla && !instance) {
			if (installingServerProjects.value.includes(projectId)) return
			startInstallingServer(projectId)
			try {
				const path = await createVanillaInstance(
					project,
					recommendedGameVersion,
					serverAddress,
				)
				if (path) {
					instance = await get(path)
					showModpackInstallSuccess(instance, serverAddress)
				}
			} finally {
				stopInstallingServer(projectId)
			}
			return
		}
		if (isModpack && !instance) {
			installToPlayModalRef?.show(projectV3, modpackVersionId, async () => {
				const newInstance = await findInstalledInstance(project.id)
				if (!newInstance) return
				await syncServerAsWorld(
					newInstance.path,
					project.title,
					serverAddress,
					project.id,
				)
				showModpackInstallSuccess(newInstance, serverAddress)
			})
			return
		}

		if (!instance) return

		await syncServerAsWorld(instance.path, project.title, serverAddress, project.id)

		// Update existing instance if needed
		if (isModpack && instance.linked_data?.version_id !== modpackVersionId) {
			updateToPlayModalRef?.show(instance, modpackVersionId, () => {
				showUpdateSuccess(instance, serverAddress)
			})
			return
		}
		if (isVanilla && instance.game_version !== recommendedGameVersion) {
			if (installingServerProjects.value.includes(projectId)) return
			startInstallingServer(projectId)
			try {
				await updateVanillaGameVersion(instance, recommendedGameVersion)
				showUpdateSuccess(instance, serverAddress)
			} finally {
				stopInstallingServer(projectId)
			}
			return
		}

		// Join server
		try {
			await joinServer(instance.path, serverAddress)
		} catch (err) {
			handleSevereError(err, { profilePath: instance.path })
		}
	}

	return {
		installingServerProjects,
		startInstallingServer,
		stopInstallingServer,
		isServerInstalling,
		installServerProject,
		playServerProject,
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
}
