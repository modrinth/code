import type { AbstractModrinthClient, Labrinth, SharedInstances } from '@modrinth/api-client'
import type { ContentItem, ContentOwner } from '@modrinth/ui'

type SharedInstanceVersionDependency = Labrinth.Versions.v2.Dependency & {
	project_id?: string
	version_id?: string
}

export function createSharedInstanceContentLoader(
	client: AbstractModrinthClient,
	fetchVersion: (
		instanceId: string,
		versionNumber: number,
	) => Promise<SharedInstances.Instances.v1.InstanceVersion> = (instanceId, versionNumber) =>
		client.sharedinstances.instances_v1.getVersion(instanceId, versionNumber),
) {
	const instanceVersions = new Map<string, SharedInstances.Instances.v1.InstanceVersion>()

	async function getVersion(
		instanceId: string,
		versionNumber: number,
	): Promise<SharedInstances.Instances.v1.InstanceVersion> {
		const cacheKey = `${instanceId}:${versionNumber}`
		const cachedVersion = instanceVersions.get(cacheKey)
		if (cachedVersion) return cachedVersion

		const version = await fetchVersion(instanceId, versionNumber)
		instanceVersions.set(cacheKey, version)
		return version
	}

	async function loadVersionContent(
		instanceId: string,
		versionNumber: number,
	): Promise<ContentItem[]> {
		const instanceVersion = await getVersion(instanceId, versionNumber)
		const modpackVersionId = instanceVersion.modpack_id
		const directVersionIds = (instanceVersion.modrinth_ids ?? []).filter(
			(versionId) => versionId !== modpackVersionId,
		)
		const modpackVersion = modpackVersionId
			? await client.labrinth.versions_v2.getVersion(modpackVersionId)
			: null
		const removedVersionIds = new Set(
			instanceVersion.removed_files.flatMap((file) =>
				file.type === 'version' ? [file.version_id] : [],
			),
		)
		const removedFileNames = new Set(
			instanceVersion.removed_files.flatMap((file) =>
				file.type === 'path' ? [file.filename] : [],
			),
		)
		const modpackDependencies = (
			(modpackVersion?.dependencies ?? []) as SharedInstanceVersionDependency[]
		).filter(
			(dependency) =>
				!(dependency.version_id && removedVersionIds.has(dependency.version_id)) &&
				!(dependency.file_name && removedFileNames.has(dependency.file_name)),
		)
		const dependencyVersionIds = modpackDependencies.flatMap((dependency) =>
			dependency.version_id ? [dependency.version_id] : [],
		)
		const modpackVersionIds = new Set(dependencyVersionIds)
		const uniqueVersionIds = [...new Set([...directVersionIds, ...dependencyVersionIds])]
		const versions = uniqueVersionIds.length
			? await client.labrinth.versions_v2.getVersions(uniqueVersionIds)
			: []
		const dependencyProjectIds = modpackDependencies.flatMap((dependency) =>
			dependency.project_id ? [dependency.project_id] : [],
		)
		const projectIds = [
			...new Set([
				...versions.map((version) => version.project_id),
				...dependencyProjectIds,
				...(modpackVersion ? [modpackVersion.project_id] : []),
			]),
		]
		const projects = projectIds.length
			? await client.labrinth.projects_v2.getMultiple(projectIds)
			: []
		const versionsById = new Map(versions.map((version) => [version.id, version]))
		const projectsById = new Map(projects.map((project) => [project.id, project]))
		const modpackProject = modpackVersion ? projectsById.get(modpackVersion.project_id) : undefined

		const directVersions = [...new Set(directVersionIds)].flatMap((versionId) => {
			const version = versionsById.get(versionId)
			return version && !modpackVersionIds.has(versionId) ? [version] : []
		})
		const ownersByProjectId = await loadProjectOwners(
			client,
			directVersions.flatMap((version) => {
				const project = projectsById.get(version.project_id)
				return project ? [project] : []
			}),
		)

		const directContent: ContentItem[] = directVersions.map((version) => {
			const project = projectsById.get(version.project_id)
			return {
				...sharedInstanceContentItem(version, project),
				owner: ownersByProjectId.get(version.project_id),
			}
		})

		const modpackContent: ContentItem[] = modpackDependencies.map((dependency) => {
			const version = dependency.version_id ? versionsById.get(dependency.version_id) : undefined
			const project = dependency.project_id
				? projectsById.get(dependency.project_id)
				: version
					? projectsById.get(version.project_id)
					: undefined
			const primaryFile = version
				? (version.files.find((file) => file.primary) ?? version.files[0])
				: undefined
			const fileName =
				primaryFile?.filename ??
				dependency.file_name ??
				project?.title ??
				version?.name ??
				'Unknown'

			const item = sharedInstanceContentItem(
				version,
				project,
				fileName,
				dependency.project_id ?? fileName,
				!project && !version,
			)
			return modpackProject
				? {
						...item,
						source: { project: modpackProject },
					}
				: item
		})

		const externalContent: ContentItem[] = instanceVersion.external_files.map((file, index) => ({
			id: `external:${file.file_type}:${file.file_name}:${index}`,
			file_name: file.file_name,
			size: file.file_size,
			project_type: file.file_type,
			has_update: false,
			update_version_id: null,
			source_kind: 'shared_instance',
			external: true,
			external_url: file.url,
			project: {
				id: file.file_name,
				slug: file.file_name,
				title: file.file_name,
				icon_url: undefined,
			},
		}))

		return [...externalContent, ...modpackContent, ...directContent]
	}

	return { getVersion, loadVersionContent }
}

async function loadProjectOwners(
	client: AbstractModrinthClient,
	projects: Labrinth.Projects.v2.Project[],
): Promise<Map<string, ContentOwner>> {
	const organizationIds = [
		...new Set(projects.flatMap((project) => (project.organization ? [project.organization] : []))),
	]
	const teamIds = [
		...new Set(projects.flatMap((project) => (project.organization ? [] : [project.team]))),
	]
	const [organizations, teams] = await Promise.all([
		organizationIds.length
			? client.labrinth.organizations_v3.getMultiple(organizationIds).catch(() => [])
			: Promise.resolve([]),
		teamIds.length
			? client.labrinth.teams_v3.getMultiple(teamIds).catch(() => [])
			: Promise.resolve([]),
	])
	const organizationsById = new Map(
		organizations.map((organization) => [organization.id, organization]),
	)
	const teamOwners = new Map(
		teams.flatMap((members) => {
			const owner = members.find((member) => member.is_owner) ?? members[0]
			return owner ? [[owner.team_id, owner.user] as const] : []
		}),
	)

	const owners = new Map<string, ContentOwner>()
	for (const project of projects) {
		const organization = project.organization
			? organizationsById.get(project.organization)
			: undefined
		const user = project.organization ? undefined : teamOwners.get(project.team)
		if (organization) {
			owners.set(project.id, {
				id: organization.slug,
				name: organization.name,
				avatar_url: organization.icon_url ?? undefined,
				type: 'organization',
			})
		} else if (user) {
			owners.set(project.id, {
				id: user.username,
				name: user.username,
				avatar_url: user.avatar_url ?? undefined,
				type: 'user',
			})
		}
	}
	return owners
}

function sharedInstanceContentItem(
	version: Labrinth.Versions.v2.Version | undefined,
	project: Labrinth.Projects.v2.Project | undefined,
	fallbackFileName?: string,
	fallbackProjectId = version?.project_id ?? fallbackFileName ?? 'unknown',
	external = false,
): ContentItem {
	const primaryFile = version
		? (version.files.find((file) => file.primary) ?? version.files[0])
		: undefined
	const fileName =
		primaryFile?.filename ?? fallbackFileName ?? project?.title ?? version?.name ?? 'Unknown'

	return {
		id: version?.id ?? project?.id ?? fileName,
		file_name: fileName,
		size: primaryFile?.size,
		project_type: project?.project_type ?? 'mod',
		has_update: false,
		update_version_id: null,
		source_kind: 'shared_instance',
		external,
		project: {
			id: project?.id ?? fallbackProjectId,
			slug: project?.slug ?? fallbackProjectId,
			title: project?.title ?? version?.name ?? fileName,
			icon_url: project?.icon_url ?? undefined,
		},
		...(version
			? {
					version: {
						id: version.id,
						version_number: version.version_number,
						file_name: fileName,
						date_published: version.date_published,
					},
				}
			: {}),
	}
}
