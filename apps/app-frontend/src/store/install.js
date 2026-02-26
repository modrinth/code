import dayjs from 'dayjs'
import { defineStore } from 'pinia'

import { trackEvent } from '@/helpers/analytics'
import { get_project, get_project_v3, get_version, get_version_many } from '@/helpers/cache.js'
import {
	create_profile_and_install as packInstall,
	install_to_existing_profile,
} from '@/helpers/pack.js'
import {
	add_project_from_version,
	check_installed,
	create,
	edit,
	edit_icon,
	get,
	get_projects,
	install as installProfile,
	list,
	remove_project,
} from '@/helpers/profile.js'
import { add_server_to_profile, get_profile_worlds, start_join_server } from '@/helpers/worlds.ts'
import router from '@/routes.js'
import { handleSevereError } from '@/store/error.js'

export const useInstall = defineStore('installStore', {
	state: () => ({
		installConfirmModal: null,
		modInstallModal: null,
		incompatibilityWarningModal: null,
		installToPlayModal: null,
		updateToPlayModal: null,
		popupNotificationManager: null,
		installingServerProjects: [],
	}),
	actions: {
		setInstallConfirmModal(ref) {
			this.installConfirmModal = ref
		},
		showInstallConfirmModal(project, version_id, onInstall, createInstanceCallback) {
			this.installConfirmModal.show(project, version_id, onInstall, createInstanceCallback)
		},
		setIncompatibilityWarningModal(ref) {
			this.incompatibilityWarningModal = ref
		},
		showIncompatibilityWarningModal(instance, project, versions, selected, onInstall) {
			this.incompatibilityWarningModal.show(instance, project, versions, selected, onInstall)
		},
		setModInstallModal(ref) {
			this.modInstallModal = ref
		},
		showModInstallModal(project, versions, onInstall) {
			this.modInstallModal.show(project, versions, onInstall)
		},
		setInstallToPlayModal(ref) {
			this.installToPlayModal = ref
		},
		showInstallToPlayModal(projectV3, modpackVersionId, onInstallComplete) {
			this.installToPlayModal.show(projectV3, modpackVersionId, onInstallComplete)
		},
		setUpdateToPlayModal(ref) {
			this.updateToPlayModal = ref
		},
		showUpdateToPlayModal(instance, activeVersionId, onUpdateComplete) {
			this.updateToPlayModal.show(instance, activeVersionId, onUpdateComplete)
		},
		setPopupNotificationManager(manager) {
			this.popupNotificationManager = manager
		},
		startInstallingServer(projectId) {
			if (!this.installingServerProjects.includes(projectId)) {
				this.installingServerProjects.push(projectId)
			}
		},
		stopInstallingServer(projectId) {
			this.installingServerProjects = this.installingServerProjects.filter((id) => id !== projectId)
		},
		isServerInstalling(projectId) {
			return this.installingServerProjects.includes(projectId)
		},
	},
})

export const findPreferredVersion = (versions, project, instance) => {
	// When `project` is passed in from this stack trace:
	// - `installVersionDependencies`
	// - `install.js/install` - `installVersionDependencies` call
	//
	// ..then `project` is actually a `Dependency` struct of a cached `Version`.
	// `Dependency` does not have a `project_type` field,
	// so we default it to `mod`.
	//
	// If we don't default here, then this `.find` will ignore version/instance
	// loader mismatches, and you'll end up e.g. installing NeoForge mods for a
	// Fabric instance.
	const projectType = project.project_type ?? 'mod'

	// If we can find a version using strictly the instance loader then prefer that
	let version = versions.find(
		(v) =>
			v.game_versions.includes(instance.game_version) &&
			(projectType === 'mod' ? v.loaders.includes(instance.loader) : true),
	)

	if (!version) {
		// Otherwise use first compatible version (in addition to versions with the instance loader this includes datapacks)
		version = versions.find((v) => isVersionCompatible(v, project, instance))
	}

	return version
}

export const isVersionCompatible = (version, project, instance) => {
	return (
		version.game_versions.includes(instance.game_version) &&
		(project.project_type === 'mod'
			? version.loaders.includes(instance.loader) || version.loaders.includes('datapack')
			: true)
	)
}

export const install = async (
	projectId,
	versionId,
	instancePath,
	source,
	callback = () => {},
	createInstanceCallback = () => {},
) => {
	const project = await get_project(projectId, 'must_revalidate')
	const projectV3 = await get_project_v3(projectId, 'must_revalidate')

	if (project.project_type === 'modpack' || projectV3?.minecraft_server != null) {
		const version = versionId ?? project.versions[project.versions.length - 1]
		const packs = await list()

		if (packs.length === 0 || !packs.find((pack) => pack.linked_data?.project_id === project.id)) {
			await packInstall(
				project.id,
				version,
				project.title,
				project.icon_url,
				createInstanceCallback,
			)

			trackEvent('PackInstall', {
				id: project.id,
				version_id: version,
				title: project.title,
				source,
			})

			callback(version)
		} else {
			const install = useInstall()
			install.showInstallConfirmModal(project, version, callback, createInstanceCallback)
		}
	} else {
		if (instancePath) {
			const [instance, instanceProjects, versions] = await Promise.all([
				await get(instancePath),
				await get_projects(instancePath),
				await get_version_many(project.versions, 'must_revalidate'),
			])

			const projectVersions = versions.sort(
				(a, b) => dayjs(b.date_published) - dayjs(a.date_published),
			)

			let version
			if (versionId) {
				version = projectVersions.find((v) => v.id === versionId)
			} else {
				version = findPreferredVersion(projectVersions, project, instance)
			}

			if (!version) {
				version = projectVersions[0]
			}

			if (isVersionCompatible(version, project, instance, true)) {
				for (const [path, file] of Object.entries(instanceProjects)) {
					if (file.metadata && file.metadata.project_id === project.id) {
						await remove_project(instance.path, path)
					}
				}

				await add_project_from_version(instance.path, version.id)
				await installVersionDependencies(instance, version)

				trackEvent('ProjectInstall', {
					loader: instance.loader,
					game_version: instance.game_version,
					id: project.id,
					project_type: project.project_type,
					version_id: version.id,
					title: project.title,
					source,
				})

				callback(version.id)
			} else {
				const install = useInstall()
				install.showIncompatibilityWarningModal(
					instance,
					project,
					projectVersions,
					version,
					callback,
				)
			}
		} else {
			let versions = (await get_version_many(project.versions)).sort(
				(a, b) => dayjs(b.date_published) - dayjs(a.date_published),
			)

			if (versionId) {
				versions = versions.filter((v) => v.id === versionId)
			}

			const install = useInstall()
			install.showModInstallModal(project, versions, callback)
		}
	}

	// If project is modpack:
	//   - We check all available instances if modpack is already installed
	//     If true: show confirmation modal
	//     If false: install it (latest version if passed version is null)
	// If project is mod:
	//   - If instance is selected:
	//        - If project is already installed
	//          We first uninstall the project
	//        - If no version is selected, we look check the instance for versions to select based on the versions
	//            - If there are no versions, we show the incompat modal
	//        - If a version is selected, and the version is incompatible, we show the incompat modal
	//   - Version is installed, as well as version dependencies
}

export const installVersionDependencies = async (profile, version) => {
	for (const dep of version.dependencies) {
		if (dep.dependency_type !== 'required') continue
		// disallow fabric api install on quilt
		if (dep.project_id === 'P7dR8mSH' && profile.loader === 'quilt') continue
		if (dep.version_id) {
			if (dep.project_id && (await check_installed(profile.path, dep.project_id))) continue
			await add_project_from_version(profile.path, dep.version_id)
		} else {
			if (dep.project_id && (await check_installed(profile.path, dep.project_id))) continue

			const depProject = await get_project(dep.project_id, 'must_revalidate')

			const depVersions = (await get_version_many(depProject.versions, 'must_revalidate')).sort(
				(a, b) => dayjs(b.date_published) - dayjs(a.date_published),
			)

			const latest = findPreferredVersion(depVersions, dep, profile)
			if (latest) {
				await add_project_from_version(profile.path, latest.id)
			}
		}
	}
}

/**
 * Server projects that use modpack content use have linked_data.project_id as
 * the server project id and linked_data.version_id as the modpack version id
 */
export const installServerProject = async (serverProjectId) => {
	const [project, projectV3] = await Promise.all([
		get_project(serverProjectId, 'must_revalidate'),
		get_project_v3(serverProjectId, 'must_revalidate'),
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

	await install_to_existing_profile(contentProjectId, contentVersionId, project.title, profilePath)

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

	await addServerAsWorld(profilePath, project.title, serverAddress)
}

const getServerAddress = (javaServer) => {
	if (!javaServer) return null
	const { address, port } = javaServer
	return port !== 25565 ? `${address}:${port}` : address
}

const addServerAsWorld = async (profilePath, serverName, serverAddress) => {
	if (!profilePath || !serverAddress) return
	try {
		const worlds = await get_profile_worlds(profilePath)
		const alreadyExists = worlds.some((w) => w.type === 'server' && w.address === serverAddress)
		if (!alreadyExists) {
			await add_server_to_profile(profilePath, serverName, serverAddress, 'prompt')
		}
	} catch (err) {
		console.error('Failed to add server to instance worlds:', err)
	}
}

const joinServer = async (profilePath, serverAddress, serverProjectId) => {
	if (!serverAddress) return
	await start_join_server(profilePath, serverAddress)
}

const findInstalledInstance = async (projectId) => {
	const packs = await list()
	return packs.find((pack) => pack.linked_data?.project_id === projectId) ?? null
}

const createVanillaInstance = async (project, gameVersion, serverAddress) => {
	const profilePath = await create(
		project.title,
		gameVersion,
		'vanilla',
		null,
		project.icon_url,
		false,
		{
			project_id: project.id,
			version_id: '',
			locked: true,
		},
	)

	await addServerAsWorld(profilePath, project.title, serverAddress)

	return profilePath
}

const updateVanillaGameVersion = async (instance, targetGameVersion) => {
	if (instance.game_version === targetGameVersion) return

	await edit(instance.path, { game_version: targetGameVersion })
	await installProfile(instance.path, false)
}

const showModpackInstallSuccess = (installStore, project, serverAddress) => {
	installStore.popupNotificationManager?.addPopupNotification({
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
									await joinServer(
										project.path,
										serverAddress,
										project.linked_data?.project_id ?? null,
									)
								} catch (err) {
									handleSevereError(err, { profilePath: project.path })
								}
							},
							color: 'brand',
						},
					]
				: []),
			{
				label: 'Instance',
				action: () => router.push(`/instance/${encodeURIComponent(project.path)}`),
			},
		],
		autoCloseMs: null,
	})
}

const showUpdateSuccess = (installStore, instance, serverAddress) => {
	installStore.popupNotificationManager?.addPopupNotification({
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
									if (serverAddress) await start_join_server(instance.path, serverAddress)
								} catch (err) {
									handleSevereError(err, { profilePath: instance.path })
								}
							},
							color: 'brand',
						},
					]
				: []),
			{
				label: 'Instance',
				action: () => router.push(`/instance/${encodeURIComponent(instance.path)}`),
			},
		],
		autoCloseMs: null,
	})
}

/**
 * Handles logic when clicking "Play" on a server project. This includes:
 * - Checking if need to install modpack content. If so, opens install to play modal
 * - Checking if need to update modpack content. If so, open update to play modal
 * - Checking if need to create instance for vanilla server. If so, creates instance.
 * - Adding server to worlds list if not already there
 * - Joining server
 */
export const playServerProject = async (projectId) => {
	const installStore = useInstall()

	const [project, projectV3] = await Promise.all([
		get_project(projectId, 'must_revalidate'),
		get_project_v3(projectId, 'must_revalidate'),
	])

	const content = projectV3?.minecraft_java_server?.content
	const serverAddress = getServerAddress(projectV3?.minecraft_java_server)
	const isVanilla = content?.kind === 'vanilla'
	const isModpack = content?.kind === 'modpack'
	const modpackVersionId = content?.version_id ?? null
	const recommendedGameVersion = content?.recommended_game_version

	let instance = await findInstalledInstance(project.id)

	if (isVanilla && !instance) {
		if (installStore.installingServerProjects.includes(projectId)) return
		installStore.startInstallingServer(projectId)
		try {
			const path = await createVanillaInstance(project, recommendedGameVersion, serverAddress)
			if (path) {
				instance = await get(path)
				showModpackInstallSuccess(installStore, instance, serverAddress)
			}
		} finally {
			installStore.stopInstallingServer(projectId)
		}
		return
	}
	if (isModpack && !instance) {
		installStore.showInstallToPlayModal(projectV3, modpackVersionId, async () => {
			const newInstance = await findInstalledInstance(project.id)
			if (!newInstance) return
			// Ensure the server is in the worlds list after modpack install
			await addServerAsWorld(newInstance.path, project.title, serverAddress)
			showModpackInstallSuccess(installStore, newInstance, serverAddress)
		})
		return
	}

	if (!instance) return

	await addServerAsWorld(instance.path, project.title, serverAddress)

	// Update existing instance if needed
	if (isModpack && instance.linked_data?.version_id !== modpackVersionId) {
		installStore.showUpdateToPlayModal(instance, modpackVersionId, async () => {
			try {
				showUpdateSuccess(installStore, instance, serverAddress)
				await joinServer(instance.path, serverAddress, project.id)
			} catch (err) {
				handleSevereError(err, { profilePath: instance.path })
			}
		})
		return
	}
	if (isVanilla && instance.game_version !== recommendedGameVersion) {
		if (installStore.installingServerProjects.includes(projectId)) return
		installStore.startInstallingServer(projectId)
		try {
			await updateVanillaGameVersion(instance, recommendedGameVersion)
			showUpdateSuccess(installStore, instance, serverAddress)
		} finally {
			installStore.stopInstallingServer(projectId)
		}
		return
	}

	// join server
	try {
		await joinServer(instance.path, serverAddress, project.id)
	} catch (err) {
		handleSevereError(err, { profilePath: instance.path })
	}
}
