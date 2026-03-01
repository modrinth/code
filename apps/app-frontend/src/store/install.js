import dayjs from 'dayjs'

import { get_project, get_version_many } from '@/helpers/cache.js'
import { add_project_from_version, check_installed } from '@/helpers/profile.js'

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
