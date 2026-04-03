// TODO: migrate to content-install.ts DI

import dayjs from 'dayjs'

import { get_project, get_version, get_version_many } from '@/helpers/cache.js'
import { add_project_from_version, check_installed } from '@/helpers/profile.js'
import {
	add_server_to_profile,
	get_profile_worlds,
	resolveManagedServerWorld,
} from '@/helpers/worlds.ts'

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

export const installVersionDependencies = async (profile, version, onDepInstalling) => {
	const projectNames = new Map()
	const storeProjectName = (p) => {
		if (p?.id && p.title) projectNames.set(p.id, p.title)
	}

	const visitedVersions = new Set()
	const announcedProjects = new Set()
	const queuedVersionIds = new Set()
	const queuedProjectVersions = new Map()
	const queuedInstalls = []
	const installedProjectCache = new Map()

	const isProjectInstalled = async (projectId) => {
		if (!projectId) return false
		if (installedProjectCache.has(projectId)) {
			return installedProjectCache.get(projectId)
		}
		const installed = await check_installed(profile.path, projectId)
		installedProjectCache.set(projectId, installed)
		return installed
	}

	const queueInstall = async (projectId, resolvedVersion) => {
		if (!resolvedVersion?.id) return false

		const versionId = resolvedVersion.id
		const resolvedProjectId = projectId ?? resolvedVersion.project_id ?? null

		if (resolvedProjectId) {
			if (await isProjectInstalled(resolvedProjectId)) return false

			const existingVersionId = queuedProjectVersions.get(resolvedProjectId)
			if (existingVersionId && existingVersionId !== versionId) return false
			if (existingVersionId === versionId) return false
		}

		if (queuedVersionIds.has(versionId)) return false

		queuedVersionIds.add(versionId)
		if (resolvedProjectId) {
			queuedProjectVersions.set(resolvedProjectId, versionId)
		}
		queuedInstalls.push({ versionId, projectId: resolvedProjectId })
		return true
	}

	const announceDependency = async (projectId, resolvedVersion) => {
		if (!onDepInstalling || !projectId) return
		if (announcedProjects.has(projectId)) return

		const depProject = await get_project(projectId, 'bypass').catch(() => null)
		if (!depProject) return

		storeProjectName(depProject)
		onDepInstalling(depProject, resolvedVersion ?? undefined)
		announcedProjects.add(projectId)
	}

	const resolveDependency = async (dep) => {
		let depVersion = null
		let depProjectId = dep.project_id ?? null

		if (dep.version_id) {
			depVersion = await get_version(dep.version_id, 'bypass').catch(() => null)
			if (!depVersion) return null

			depProjectId = depProjectId ?? depVersion.project_id ?? null
			if (depProjectId && !projectNames.has(depProjectId)) {
				const p = await get_project(depProjectId, 'bypass').catch(() => null)
				storeProjectName(p)
			}
		} else if (dep.project_id) {
			const depProject = await get_project(dep.project_id, 'bypass').catch(() => null)
			if (!depProject) return null

			storeProjectName(depProject)

			const depVersions = await get_version_many(depProject.versions, 'bypass').catch(() => [])
			depVersion = findPreferredVersion(
				depVersions.sort((a, b) => dayjs(b.date_published) - dayjs(a.date_published)),
				dep,
				profile,
			)
			if (!depVersion) return null

			depProjectId = dep.project_id
		} else {
			return null
		}

		return { depVersion, depProjectId }
	}

	const collectDependenciesForVersion = async (inputVersion) => {
		if (!inputVersion?.id || visitedVersions.has(inputVersion.id)) return
		visitedVersions.add(inputVersion.id)

		if (inputVersion.project_id && !projectNames.has(inputVersion.project_id)) {
			const p = await get_project(inputVersion.project_id, 'bypass').catch(() => null)
			storeProjectName(p)
		}

		for (const dep of inputVersion.dependencies ?? []) {
			if (dep.dependency_type !== 'required') continue
			if (dep.project_id === 'P7dR8mSH' && profile.loader === 'quilt') continue

			const resolved = await resolveDependency(dep, inputVersion)
			if (!resolved) continue

			const { depVersion, depProjectId } = resolved
			const queued = await queueInstall(depProjectId, depVersion)
			if (queued && depProjectId) {
				await announceDependency(depProjectId, depVersion)
			}

			await collectDependenciesForVersion(depVersion)
		}
	}

	await collectDependenciesForVersion(version)

	if (queuedInstalls.length === 0) return

	const batchSize = 8
	for (let i = 0; i < queuedInstalls.length; i += batchSize) {
		const batch = queuedInstalls.slice(i, i + batchSize)
		await Promise.all(
			batch.map(async ({ versionId }) => {
				await add_project_from_version(profile.path, versionId)
			}),
		)
	}
}

export const getServerAddress = (javaServer) => {
	if (!javaServer) return null
	const { address } = javaServer
	return address
}

export const ensureManagedServerWorldExists = async (profilePath, serverName, serverAddress) => {
	if (!profilePath || !serverAddress) return
	try {
		const worlds = await get_profile_worlds(profilePath)
		const managedWorld = resolveManagedServerWorld(worlds, serverName, serverAddress)
		if (!managedWorld) {
			await add_server_to_profile(profilePath, serverName, serverAddress, 'prompt')
		}
	} catch (err) {
		console.error('Failed to ensure managed server world exists:', err)
	}
}
