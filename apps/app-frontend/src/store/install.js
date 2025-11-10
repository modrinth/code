import dayjs from 'dayjs'
import { defineStore } from 'pinia'

import { trackEvent } from '@/helpers/analytics.js'
import { get_project, get_version_many } from '@/helpers/cache.js'
import { create_profile_and_install as packInstall } from '@/helpers/pack.js'
import {
	add_project_from_version,
	check_installed,
	get,
	get_projects,
	list,
	remove_project,
} from '@/helpers/profile.js'

export const useInstall = defineStore('installStore', {
	state: () => ({
		installConfirmModal: null,
		modInstallModal: null,
		incompatibilityWarningModal: null,
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

	if (project.project_type === 'modpack') {
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
