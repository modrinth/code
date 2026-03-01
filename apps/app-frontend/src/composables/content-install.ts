import type { ContentInstallInstance } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import dayjs from 'dayjs'
import { nextTick, ref } from 'vue'
import type { Router } from 'vue-router'

import { trackEvent } from '@/helpers/analytics'
import { get_project, get_version_many } from '@/helpers/cache.js'
import { create_profile_and_install as packInstall } from '@/helpers/pack'
import {
	add_project_from_version,
	check_installed,
	create,
	get,
	get_projects,
	list,
	remove_project,
} from '@/helpers/profile.js'
import { findPreferredVersion, installVersionDependencies, isVersionCompatible } from '@/store/install.js'
import { get_game_versions } from '@/helpers/tags'

const LOADER_ORDER = ['vanilla', 'fabric', 'quilt', 'neoforge', 'forge']
const SUPPORTED_LOADERS: Set<string> = new Set(['vanilla', 'forge', 'fabric', 'quilt', 'neoforge'])
// These loaders indicate content that works on vanilla instances
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

const instances = ref<ContentInstallInstance[]>([])
const compatibleLoaders = ref<string[]>([])
const gameVersions = ref<string[]>([])
const loading = ref(false)
const defaultTab = ref<'existing' | 'new'>('existing')
const preferredLoader = ref<string | null>(null)
const preferredGameVersion = ref<string | null>(null)
const releaseGameVersions = ref<Set<string>>(new Set())

let modalRef: { show: () => void; hide: () => void } | null = null
let installConfirmModalRef: any = null
let incompatibilityWarningModalRef: any = null
let currentProject: any = null
let currentVersions: any[] = []
let currentCallback: (versionId?: string) => void = () => {}
let profileMap: Record<string, any> = {}
let routerInstance: Router | null = null
let handleErrorFn: (err: any) => void = console.error

export function setupContentInstall(opts: {
	router: Router
	handleError: (err: any) => void
}) {
	routerInstance = opts.router
	handleErrorFn = opts.handleError

	return {
		contentInstallInstances: instances,
		contentInstallLoaders: compatibleLoaders,
		contentInstallGameVersions: gameVersions,
		contentInstallLoading: loading,
		contentInstallDefaultTab: defaultTab,
		contentInstallPreferredLoader: preferredLoader,
		contentInstallPreferredGameVersion: preferredGameVersion,
		contentInstallReleaseGameVersions: releaseGameVersions,
		handleInstallToInstance,
		handleCreateAndInstall,
		handleCancel,
		setContentInstallModal(ref: any) {
			modalRef = ref
		},
		setInstallConfirmModal(ref: any) {
			installConfirmModalRef = ref
		},
		setIncompatibilityWarningModal(ref: any) {
			incompatibilityWarningModalRef = ref
		},
	}
}

async function showModInstallModal(
	project: any,
	versions: any[],
	onInstall: (versionId?: string) => void,
	hints?: { preferredLoader?: string; preferredGameVersion?: string },
) {
	currentProject = project
	currentVersions = versions
	currentCallback = onInstall

	instances.value = []
	loading.value = true
	defaultTab.value = 'existing'

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
		hints?.preferredLoader && loaderSet.has(hints.preferredLoader)
			? hints.preferredLoader
			: null
	preferredGameVersion.value =
		hints?.preferredGameVersion && gameVersionSet.has(hints.preferredGameVersion)
			? hints.preferredGameVersion
			: null

	await nextTick()
	modalRef?.show()
	trackEvent('ProjectInstallStart', { source: 'ProjectInstallModal' })

	try {
		const profiles = await list()
		const newProfileMap: Record<string, any> = {}
		const installedChecks = await Promise.all(
			profiles.map((profile: any) => check_installed(profile.path, project.id)),
		)

		const newInstances: ContentInstallInstance[] = profiles.map(
			(profile: any, i: number) => {
				newProfileMap[profile.path] = profile
				return {
					id: profile.path,
					name: profile.name,
					iconUrl: profile.icon_path ? convertFileSrc(profile.icon_path) : null,
					installed: installedChecks[i],
					compatible: versions.some((v: any) =>
						isVersionCompatible(v, project, profile),
					),
					installing: false,
				}
			},
		)

		profileMap = newProfileMap
		instances.value = newInstances

		if (!newInstances.some((i) => i.compatible && !i.installed)) {
			defaultTab.value = 'new'
		}
	} catch (err) {
		handleErrorFn(err)
	} finally {
		loading.value = false
	}
}

async function handleInstallToInstance(instance: ContentInstallInstance) {
	const profile = profileMap[instance.id]
	const storeInstance = instances.value.find((i) => i.id === instance.id)
	if (storeInstance) storeInstance.installing = true

	const version = findPreferredVersion(currentVersions, currentProject, profile)
	if (!version) {
		if (storeInstance) storeInstance.installing = false
		handleErrorFn('No compatible version found')
		return
	}

	try {
		await add_project_from_version(instance.id, version.id)
		await installVersionDependencies(profile, version)
		if (storeInstance) {
			storeInstance.installed = true
			storeInstance.installing = false
		}
		trackEvent('ProjectInstall', {
			loader: profile.loader,
			game_version: profile.game_version,
			id: currentProject.id,
			version_id: version.id,
			project_type: currentProject.project_type,
			title: currentProject.title,
			source: 'ProjectInstallModal',
		})
		currentCallback(version.id)
		modalRef?.hide()
		await routerInstance!.push(`/instance/${encodeURIComponent(instance.id)}/`)
	} catch (err) {
		if (storeInstance) storeInstance.installing = false
		handleErrorFn(err)
	}
}

async function handleCreateAndInstall(data: {
	name: string
	iconPath: string | null
	iconPreviewUrl: string | null
	loader: string
	gameVersion: string
}) {
	// When creating a vanilla instance for datapacks/resource packs, match versions
	// with "datapack" or "minecraft" loaders since those work on vanilla instances.
	const loaderCandidates =
		data.loader === 'vanilla'
			? ['vanilla', 'datapack', 'minecraft']
			: [data.loader]
	const version =
		currentVersions.find(
			(v: any) =>
				v.game_versions.includes(data.gameVersion) &&
				loaderCandidates.some((l) => v.loaders.includes(l)),
		) ?? currentVersions[0]

	try {
		const id = await create(
			data.name,
			data.gameVersion,
			data.loader as any,
			'latest',
			data.iconPath,
			false,
		)
		if (!id) return

		await add_project_from_version(id, version.id)
		await routerInstance!.push(`/instance/${encodeURIComponent(id)}/`)

		const instance = await get(id)
		await installVersionDependencies(instance, version)

		trackEvent('InstanceCreate', {
			source: 'ProjectInstallModal',
		})
		trackEvent('ProjectInstall', {
			loader: data.loader,
			game_version: data.gameVersion,
			id: currentProject.id,
			version_id: version.id,
			project_type: currentProject.project_type,
			title: currentProject.title,
			source: 'ProjectInstallModal',
		})

		currentCallback(version.id)
		modalRef?.hide()
	} catch (err) {
		handleErrorFn(err)
	}
}

function handleCancel() {
	currentCallback?.()
}

export async function install(
	projectId: string,
	versionId?: string | null,
	instancePath?: string | null,
	source: string = 'unknown',
	callback: (versionId?: string) => void = () => {},
	createInstanceCallback: (profile: string) => void = () => {},
	hints?: { preferredLoader?: string; preferredGameVersion?: string },
) {
	const project = await get_project(projectId, 'must_revalidate')

	if (project.project_type === 'modpack') {
		const version = versionId ?? project.versions[project.versions.length - 1]
		const packs = await list()

		if (
			packs.length === 0 ||
			!packs.find((pack: any) => pack.linked_data?.project_id === project.id)
		) {
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
			installConfirmModalRef?.show(project, version, callback, createInstanceCallback)
		}
	} else if (instancePath) {
		const [instanceOrNull, instanceProjects, versions] = await Promise.all([
			get(instancePath),
			get_projects(instancePath),
			get_version_many(project.versions, 'must_revalidate'),
		])
		if (!instanceOrNull) return

		const instance = instanceOrNull
		const projectVersions = versions.sort(
			(a: any, b: any) =>
				dayjs(b.date_published).valueOf() - dayjs(a.date_published).valueOf(),
		)

		let version = versionId
			? projectVersions.find((v: any) => v.id === versionId)
			: findPreferredVersion(projectVersions, project, instance)
		if (!version) version = projectVersions[0]

		if (isVersionCompatible(version, project, instance)) {
			for (const [path, file] of Object.entries(instanceProjects) as [string, any][]) {
				if (file.metadata?.project_id === project.id) {
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
			incompatibilityWarningModalRef?.show(
				instance,
				project,
				projectVersions,
				version,
				callback,
			)
		}
	} else {
		let versions = (await get_version_many(project.versions)).sort(
			(a: any, b: any) =>
				dayjs(b.date_published).valueOf() - dayjs(a.date_published).valueOf(),
		)
		if (versionId) versions = versions.filter((v: any) => v.id === versionId)
		await showModInstallModal(project, versions, callback, hints)
	}
}
