import type { Labrinth } from '@modrinth/api-client'
import type { ContentInstallInstance, ContentInstallProjectInfo, ContentItem } from '@modrinth/ui'
import { createContext } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import dayjs from 'dayjs'
import { nextTick, type Ref, ref } from 'vue'
import type { Router } from 'vue-router'

import { trackEvent } from '@/helpers/analytics'
import {
	get_organization,
	get_project,
	get_project_v3_many,
	get_team,
	get_version_many,
} from '@/helpers/cache.js'
import { create_profile_and_install as packInstall } from '@/helpers/pack'
import {
	add_project_from_version,
	check_installed_batch,
	create,
	get,
	get_projects,
	list,
	remove_project,
} from '@/helpers/profile.js'
import { get_game_versions } from '@/helpers/tags'
import type { GameInstance, InstanceLoader } from '@/helpers/types'
import {
	findPreferredVersion,
	installVersionDependencies,
	isVersionCompatible,
} from '@/store/install.js'

interface ModalRef {
	show: () => void
	hide: () => void
}

interface ModpackAlreadyInstalledModalRef {
	show: (instanceName: string, instancePath: string) => void
}

interface IncompatibilityWarningModalRef {
	show: (
		instance: GameInstance,
		project: Labrinth.Projects.v2.Project,
		versions: Labrinth.Versions.v2.Version[],
		version: Labrinth.Versions.v2.Version,
		callback: (versionId?: string) => void,
	) => void
}

const LOADER_ORDER = ['vanilla', 'fabric', 'quilt', 'neoforge', 'forge']
const SUPPORTED_LOADERS: Set<string> = new Set(['vanilla', 'forge', 'fabric', 'quilt', 'neoforge'])
const VANILLA_COMPATIBLE_LOADERS: Set<string> = new Set(['minecraft', 'datapack'])

function sortLoaders(loaders: string[]): string[] {
	return loaders.slice().sort((a, b) => {
		const aIdx = LOADER_ORDER.indexOf(a)
		const bIdx = LOADER_ORDER.indexOf(b)
		if (aIdx === -1 && bIdx === -1) return a.localeCompare(b)
		if (aIdx === -1) return 1
		if (bIdx === -1) return -1
		return aIdx - bIdx
	})
}

export interface ContentInstallContext {
	instances: Ref<ContentInstallInstance[]>
	compatibleLoaders: Ref<string[]>
	gameVersions: Ref<string[]>
	loading: Ref<boolean>
	defaultTab: Ref<'existing' | 'new'>
	preferredLoader: Ref<string | null>
	preferredGameVersion: Ref<string | null>
	releaseGameVersions: Ref<Set<string>>
	projectInfo: Ref<ContentInstallProjectInfo | null>
	handleInstallToInstance: (instance: ContentInstallInstance) => Promise<void>
	handleCreateAndInstall: (data: {
		name: string
		iconPath: string | null
		iconPreviewUrl: string | null
		loader: string
		gameVersion: string
	}) => Promise<void>
	handleNavigate: (instance: ContentInstallInstance) => void
	handleCancel: () => void
	setContentInstallModal: (ref: ModalRef) => void
	setModpackAlreadyInstalledModal: (ref: ModpackAlreadyInstalledModalRef) => void
	handleModpackDuplicateCreateAnyway: () => Promise<void>
	handleModpackDuplicateGoToInstance: (instancePath: string) => void
	setIncompatibilityWarningModal: (ref: IncompatibilityWarningModalRef) => void
	install: (
		projectId: string,
		versionId?: string | null,
		instancePath?: string | null,
		source?: string,
		callback?: (versionId?: string) => void,
		createInstanceCallback?: (profile: string) => void,
		hints?: { preferredLoader?: string; preferredGameVersion?: string; showProjectInfo?: boolean },
	) => Promise<void>
	installingItems: Ref<Map<string, ContentItem[]>>
}

export const [injectContentInstall, provideContentInstall] = createContext<ContentInstallContext>(
	'root',
	'contentInstall',
)

export function createContentInstall(opts: {
	router: Router
	handleError: (err: unknown) => void
}): ContentInstallContext {
	const instances = ref<ContentInstallInstance[]>([])
	const compatibleLoaders = ref<string[]>([])
	const gameVersions = ref<string[]>([])
	const loading = ref(false)
	const defaultTab = ref<'existing' | 'new'>('existing')
	const preferredLoader = ref<string | null>(null)
	const preferredGameVersion = ref<string | null>(null)
	const releaseGameVersions = ref<Set<string>>(new Set())

	const projectInfo = ref<ContentInstallProjectInfo | null>(null)
	const installingItems = ref<Map<string, ContentItem[]>>(new Map())

	function addInstallingItem(
		instancePath: string,
		project: {
			id: string
			slug?: string | null
			title: string
			icon_url?: string | null
			project_type?: string
		},
		version?: Labrinth.Versions.v2.Version,
	) {
		const primaryFile = version?.files?.find((f) => f.primary) ?? version?.files?.[0]
		const placeholder: ContentItem = {
			id: `__installing_${project.id}`,
			file_name: `__installing_${project.id}`,
			project: {
				id: project.id,
				slug: project.slug ?? '',
				title: project.title,
				icon_url: project.icon_url ?? undefined,
			},
			version: version
				? {
						id: version.id,
						version_number: version.version_number,
						file_name: primaryFile?.filename ?? '',
					}
				: undefined,
			project_type: project.project_type ?? 'mod',
			has_update: false,
			update_version_id: null,
			enabled: true,
			installing: true,
		}
		const next = new Map(installingItems.value)
		const items = next.get(instancePath) ?? []
		if (items.some((i) => i.file_name === placeholder.file_name)) return
		next.set(instancePath, [...items, placeholder])
		installingItems.value = next
	}

	function removeInstallingItems(instancePath: string, projectIds: string[]) {
		const next = new Map(installingItems.value)
		const items = next.get(instancePath)
		if (items) {
			const idsToRemove = new Set(projectIds.map((id) => `__installing_${id}`))
			const filtered = items.filter((i) => !idsToRemove.has(i.file_name))
			if (filtered.length > 0) {
				next.set(instancePath, filtered)
			} else {
				next.delete(instancePath)
			}
			installingItems.value = next
		}
	}

	let modalRef: ModalRef | null = null
	let modpackAlreadyInstalledModalRef: ModpackAlreadyInstalledModalRef | null = null
	let incompatibilityWarningModalRef: IncompatibilityWarningModalRef | null = null
	let currentProject: Labrinth.Projects.v2.Project | null = null
	let currentVersions: Labrinth.Versions.v2.Version[] = []
	let currentCallback: (versionId?: string) => void = () => {}
	let profileMap: Record<string, GameInstance> = {}

	let pendingModpackInstall: {
		project: Labrinth.Projects.v2.Project
		version: string
		source: string
		callback: (versionId?: string) => void
		createInstanceCallback: (profile: string) => void
	} | null = null

	async function showModInstallModal(
		project: Labrinth.Projects.v2.Project,
		versions: Labrinth.Versions.v2.Version[],
		onInstall: (versionId?: string) => void,
		hints?: { preferredLoader?: string; preferredGameVersion?: string; showProjectInfo?: boolean },
	) {
		currentProject = project
		currentVersions = versions
		currentCallback = onInstall

		instances.value = []
		defaultTab.value = 'existing'

		if (hints?.showProjectInfo) {
			projectInfo.value = {
				title: project.title,
				iconUrl: project.icon_url,
				link: `/project/${project.slug ?? project.id}`,
			}
			if (project.organization) {
				get_organization(project.organization)
					.then((org: { id: string; slug: string; name: string; icon_url?: string }) => {
						if (projectInfo.value) {
							const orgSlug = org.slug ?? org.id
							projectInfo.value = {
								...projectInfo.value,
								owner: {
									name: org.name,
									iconUrl: org.icon_url,
									circle: false,
									link: () => openUrl(`https://modrinth.com/organization/${orgSlug}`),
								},
							}
						}
					})
					.catch(() => {})
			} else if (project.team) {
				get_team(project.team)
					.then(
						(
							members: {
								user: { id: string; username: string; avatar_url?: string }
								is_owner: boolean
							}[],
						) => {
							const owner = members.find((m) => m.is_owner)
							if (owner && projectInfo.value) {
								projectInfo.value = {
									...projectInfo.value,
									owner: {
										name: owner.user.username,
										iconUrl: owner.user.avatar_url,
										circle: true,
										link: () => openUrl(`https://modrinth.com/user/${owner.user.username}`),
									},
								}
							}
						},
					)
					.catch(() => {})
			}
		} else {
			projectInfo.value = null
		}

		const loaderSet = new Set<string>()
		const gameVersionSet = new Set<string>()
		for (const v of versions) {
			for (const l of v.loaders) loaderSet.add(l)
			for (const gv of v.game_versions) gameVersionSet.add(gv)
		}
		const mappedLoaders = new Set<string>()
		for (const l of loaderSet) {
			if (SUPPORTED_LOADERS.has(l)) mappedLoaders.add(l)
			else if (VANILLA_COMPATIBLE_LOADERS.has(l)) mappedLoaders.add('vanilla')
		}
		compatibleLoaders.value = sortLoaders([...mappedLoaders])

		try {
			const allGameVersions = await get_game_versions()
			const releases = new Set<string>()
			const ordered: string[] = []
			for (const gv of allGameVersions) {
				if (gameVersionSet.has(gv.version)) {
					ordered.push(gv.version)
					if (gv.version_type === 'release') {
						releases.add(gv.version)
					}
				}
			}
			gameVersions.value = ordered
			releaseGameVersions.value = releases
		} catch {
			gameVersions.value = [...gameVersionSet]
			releaseGameVersions.value = new Set(gameVersionSet)
		}

		preferredLoader.value =
			hints?.preferredLoader && loaderSet.has(hints.preferredLoader) ? hints.preferredLoader : null
		preferredGameVersion.value =
			hints?.preferredGameVersion && gameVersionSet.has(hints.preferredGameVersion)
				? hints.preferredGameVersion
				: null

		try {
			let profiles = await list()

			const linkedProjectIds = profiles
				.filter((p) => p.linked_data?.project_id)
				.map((p) => p.linked_data!.project_id)
			if (linkedProjectIds.length > 0) {
				const linkedProjects = await get_project_v3_many(linkedProjectIds, 'must_revalidate').catch(
					() => [],
				)
				const serverProjectIds = new Set(
					linkedProjects
						.filter((p: { id: string; minecraft_server?: unknown }) => p?.minecraft_server != null)
						.map((p: { id: string }) => p.id),
				)
				profiles = profiles.filter(
					(p) => !p.linked_data?.project_id || !serverProjectIds.has(p.linked_data.project_id),
				)
			}

			const newProfileMap: Record<string, GameInstance> = {}
			const installedMap = await check_installed_batch(project.id)

			const newInstances: ContentInstallInstance[] = profiles.map((profile) => {
				newProfileMap[profile.path] = profile
				return {
					id: profile.path,
					name: profile.name,
					iconUrl: profile.icon_path ? convertFileSrc(profile.icon_path) : null,
					installed: installedMap[profile.path] ?? false,
					compatible: versions.some((v) => isVersionCompatible(v, project, profile)),
					installing: false,
				}
			})

			profileMap = newProfileMap
			instances.value = newInstances

			if (!newInstances.some((i) => i.compatible && !i.installed)) {
				defaultTab.value = 'new'
			}
		} catch (err) {
			opts.handleError(err)
		}

		await nextTick()
		modalRef?.show()
		trackEvent('ProjectInstallStart', { source: 'ProjectInstallModal' })
	}

	async function handleInstallToInstance(instance: ContentInstallInstance) {
		const profile = profileMap[instance.id]
		const storeInstance = instances.value.find((i) => i.id === instance.id)
		if (storeInstance) storeInstance.installing = true

		const version = findPreferredVersion(currentVersions, currentProject, profile)
		if (!version) {
			if (storeInstance) storeInstance.installing = false
			opts.handleError('No compatible version found')
			return
		}

		const installedProjectIds: string[] = []
		if (currentProject) {
			addInstallingItem(instance.id, currentProject, version)
			installedProjectIds.push(currentProject.id)
		}

		try {
			await add_project_from_version(instance.id, version.id)
			await installVersionDependencies(
				profile,
				version,
				(depProject: Labrinth.Projects.v2.Project, depVersion?: Labrinth.Versions.v2.Version) => {
					addInstallingItem(instance.id, depProject, depVersion)
					installedProjectIds.push(depProject.id)
				},
			)
			if (storeInstance) {
				storeInstance.installed = true
				storeInstance.installing = false
			}
			trackEvent('ProjectInstall', {
				loader: profile.loader,
				game_version: profile.game_version,
				id: currentProject!.id,
				version_id: version.id,
				project_type: currentProject!.project_type,
				title: currentProject!.title,
				source: 'ProjectInstallModal',
			})
			currentCallback(version.id)
		} catch (err) {
			if (storeInstance) storeInstance.installing = false
			opts.handleError(err)
		} finally {
			removeInstallingItems(instance.id, installedProjectIds)
		}
	}

	async function handleCreateAndInstall(data: {
		name: string
		iconPath: string | null
		iconPreviewUrl: string | null
		loader: string
		gameVersion: string
	}) {
		const loaderCandidates =
			data.loader === 'vanilla' ? ['vanilla', 'datapack', 'minecraft'] : [data.loader]
		const version =
			currentVersions.find(
				(v) =>
					v.game_versions.includes(data.gameVersion) &&
					loaderCandidates.some((l) => v.loaders.includes(l)),
			) ?? currentVersions[0]

		try {
			const id = await create(
				data.name,
				data.gameVersion,
				data.loader as InstanceLoader,
				'latest',
				data.iconPath,
				false,
			)
			if (!id) return

			await add_project_from_version(id, version.id)
			await opts.router.push(`/instance/${encodeURIComponent(id)}/`)

			const instance = await get(id)
			await installVersionDependencies(instance, version)

			trackEvent('InstanceCreate', {
				source: 'ProjectInstallModal',
			})
			trackEvent('ProjectInstall', {
				loader: data.loader,
				game_version: data.gameVersion,
				id: currentProject!.id,
				version_id: version.id,
				project_type: currentProject!.project_type,
				title: currentProject!.title,
				source: 'ProjectInstallModal',
			})

			currentCallback(version.id)
			modalRef?.hide()
		} catch (err) {
			opts.handleError(err)
		}
	}

	function handleNavigate(instance: ContentInstallInstance) {
		modalRef?.hide()
		opts.router.push(`/instance/${encodeURIComponent(instance.id)}/`)
	}

	function handleCancel() {
		currentCallback?.()
	}

	async function install(
		projectId: string,
		versionId?: string | null,
		instancePath?: string | null,
		source: string = 'unknown',
		callback: (versionId?: string) => void = () => {},
		createInstanceCallback: (profile: string) => void = () => {},
		hints?: { preferredLoader?: string; preferredGameVersion?: string; showProjectInfo?: boolean },
	) {
		const project: Labrinth.Projects.v2.Project = await get_project(projectId, 'must_revalidate')

		if (project.project_type === 'modpack') {
			const version = versionId ?? project.versions[project.versions.length - 1]
			const packs = await list()
			const existingPack = packs.find((pack) => pack.linked_data?.project_id === project.id)

			if (existingPack) {
				pendingModpackInstall = { project, version, source, callback, createInstanceCallback }
				modpackAlreadyInstalledModalRef?.show(existingPack.name, existingPack.path)
				return
			}

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
		} else if (instancePath) {
			const [instanceOrNull, instanceProjects, versions] = await Promise.all([
				get(instancePath),
				get_projects(instancePath),
				get_version_many(project.versions, 'must_revalidate') as Promise<
					Labrinth.Versions.v2.Version[]
				>,
			])
			if (!instanceOrNull) return

			const instance = instanceOrNull
			const projectVersions = versions.sort(
				(a, b) => dayjs(b.date_published).valueOf() - dayjs(a.date_published).valueOf(),
			)

			let version = versionId
				? projectVersions.find((v) => v.id === versionId)
				: findPreferredVersion(projectVersions, project, instance)
			if (!version) version = projectVersions[0]

			if (isVersionCompatible(version, project, instance)) {
				for (const [path, file] of Object.entries(instanceProjects)) {
					if (file.metadata?.project_id === project.id) {
						await remove_project(instance.path, path)
					}
				}

				const installedProjectIds: string[] = [project.id]
				addInstallingItem(instancePath, project, version)
				try {
					await add_project_from_version(instance.path, version.id)
					await installVersionDependencies(
						instance,
						version,
						(
							depProject: Labrinth.Projects.v2.Project,
							depVersion?: Labrinth.Versions.v2.Version,
						) => {
							addInstallingItem(instancePath, depProject, depVersion)
							installedProjectIds.push(depProject.id)
						},
					)

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
				} finally {
					removeInstallingItems(instancePath, installedProjectIds)
				}
			} else {
				incompatibilityWarningModalRef?.show(instance, project, projectVersions, version, callback)
			}
		} else {
			let versions = (
				(await get_version_many(project.versions)) as Labrinth.Versions.v2.Version[]
			).sort((a, b) => dayjs(b.date_published).valueOf() - dayjs(a.date_published).valueOf())
			if (versionId) versions = versions.filter((v) => v.id === versionId)
			await showModInstallModal(project, versions, callback, hints)
		}
	}

	return {
		instances,
		compatibleLoaders,
		gameVersions,
		loading,
		defaultTab,
		preferredLoader,
		preferredGameVersion,
		releaseGameVersions,
		projectInfo,
		handleInstallToInstance,
		handleCreateAndInstall,
		handleNavigate,
		handleCancel,
		setContentInstallModal(ref: ModalRef) {
			modalRef = ref
		},
		setModpackAlreadyInstalledModal(ref: ModpackAlreadyInstalledModalRef) {
			modpackAlreadyInstalledModalRef = ref
		},
		async handleModpackDuplicateCreateAnyway() {
			if (!pendingModpackInstall) return
			const { project, version, source, callback, createInstanceCallback } = pendingModpackInstall
			pendingModpackInstall = null
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
		},
		handleModpackDuplicateGoToInstance(instancePath: string) {
			pendingModpackInstall = null
			opts.router.push(`/instance/${encodeURIComponent(instancePath)}/`)
		},
		setIncompatibilityWarningModal(ref: IncompatibilityWarningModalRef) {
			incompatibilityWarningModalRef = ref
		},
		install,
		installingItems,
	}
}
