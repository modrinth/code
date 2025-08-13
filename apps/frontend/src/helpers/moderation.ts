import type { ExtendedReport, OwnershipTarget } from '@modrinth/moderation'
import type {
	Organization,
	Project,
	Report,
	TeamMember,
	Thread,
	User,
	Version,
} from '@modrinth/utils'

export const useModerationCache = () => ({
	threads: useState<Map<string, Thread>>('moderation-report-cache-threads', () => new Map()),
	users: useState<Map<string, User>>('moderation-report-cache-users', () => new Map()),
	projects: useState<Map<string, Project>>('moderation-report-cache-projects', () => new Map()),
	versions: useState<Map<string, Version>>('moderation-report-cache-versions', () => new Map()),
	teams: useState<Map<string, TeamMember[]>>('moderation-report-cache-teams', () => new Map()),
	orgs: useState<Map<string, Organization>>('moderation-report-cache-orgs', () => new Map()),
})

// TODO: @AlexTMjugador - backend should do all of these functions.
export async function enrichReportBatch(reports: Report[]): Promise<ExtendedReport[]> {
	if (reports.length === 0) return []

	const cache = useModerationCache()

	const threadIDs = reports
		.map((r) => r.thread_id)
		.filter(Boolean)
		.filter((id) => !cache.threads.value.has(id))
	const userIDs = [
		...reports.filter((r) => r.item_type === 'user').map((r) => r.item_id),
		...reports.map((r) => r.reporter),
	].filter((id) => !cache.users.value.has(id))
	const versionIDs = reports
		.filter((r) => r.item_type === 'version')
		.map((r) => r.item_id)
		.filter((id) => !cache.versions.value.has(id))
	const projectIDs = reports
		.filter((r) => r.item_type === 'project')
		.map((r) => r.item_id)
		.filter((id) => !cache.projects.value.has(id))

	const [newThreads, newVersions, newUsers] = await Promise.all([
		threadIDs.length > 0
			? (fetchSegmented(threadIDs, (ids) => `threads?ids=${asEncodedJsonArray(ids)}`) as Promise<
					Thread[]
				>)
			: Promise.resolve([]),
		versionIDs.length > 0
			? (fetchSegmented(versionIDs, (ids) => `versions?ids=${asEncodedJsonArray(ids)}`) as Promise<
					Version[]
				>)
			: Promise.resolve([]),
		[...new Set(userIDs)].length > 0
			? (fetchSegmented(
					[...new Set(userIDs)],
					(ids) => `users?ids=${asEncodedJsonArray(ids)}`,
				) as Promise<User[]>)
			: Promise.resolve([]),
	])

	newThreads.forEach((t) => cache.threads.value.set(t.id, t))
	newVersions.forEach((v) => cache.versions.value.set(v.id, v))
	newUsers.forEach((u) => cache.users.value.set(u.id, u))

	const allVersions = [...newVersions, ...Array.from(cache.versions.value.values())]
	const fullProjectIds = new Set([
		...projectIDs,
		...allVersions
			.filter((v) => versionIDs.includes(v.id))
			.map((v) => v.project_id)
			.filter(Boolean),
	])

	const uncachedProjectIds = Array.from(fullProjectIds).filter(
		(id) => !cache.projects.value.has(id),
	)
	const newProjects =
		uncachedProjectIds.length > 0
			? ((await fetchSegmented(
					uncachedProjectIds,
					(ids) => `projects?ids=${asEncodedJsonArray(ids)}`,
				)) as Project[])
			: []

	newProjects.forEach((p) => cache.projects.value.set(p.id, p))

	const allProjects = [...newProjects, ...Array.from(cache.projects.value.values())]
	const teamIds = [...new Set(allProjects.map((p) => p.team).filter(Boolean))].filter(
		(id) => !cache.teams.value.has(id || 'invalid team id'),
	)
	const orgIds = [...new Set(allProjects.map((p) => p.organization).filter(Boolean))].filter(
		(id) => !cache.orgs.value.has(id),
	)

	const [newTeams, newOrgs] = await Promise.all([
		teamIds.length > 0
			? (fetchSegmented(teamIds, (ids) => `teams?ids=${asEncodedJsonArray(ids)}`) as Promise<
					TeamMember[][]
				>)
			: Promise.resolve([]),
		orgIds.length > 0
			? (fetchSegmented(orgIds, (ids) => `organizations?ids=${asEncodedJsonArray(ids)}`, {
					apiVersion: 3,
				}) as Promise<Organization[]>)
			: Promise.resolve([]),
	])

	newTeams.forEach((team) => {
		if (team.length > 0) cache.teams.value.set(team[0].team_id, team)
	})
	newOrgs.forEach((org) => cache.orgs.value.set(org.id, org))

	return reports.map((report) => {
		const thread = cache.threads.value.get(report.thread_id) || ({} as Thread)
		const version =
			report.item_type === 'version' ? cache.versions.value.get(report.item_id) : undefined

		const project =
			report.item_type === 'project'
				? cache.projects.value.get(report.item_id)
				: report.item_type === 'version' && version
					? cache.projects.value.get(version.project_id)
					: undefined

		let target: OwnershipTarget | undefined

		if (report.item_type === 'user') {
			const targetUser = cache.users.value.get(report.item_id)
			if (targetUser) {
				target = {
					name: targetUser.username,
					slug: targetUser.username,
					avatar_url: targetUser.avatar_url,
					type: 'user',
				}
			}
		} else if (project) {
			let owner: TeamMember | null = null
			let org: Organization | null = null

			if (project.team) {
				const teamMembers = cache.teams.value.get(project.team)
				if (teamMembers) {
					owner = teamMembers.find((member) => member.role === 'Owner') || null
				}
			}

			if (project.organization) {
				org = cache.orgs.value.get(project.organization) || null
			}

			if (org) {
				target = {
					name: org.name,
					avatar_url: org.icon_url,
					type: 'organization',
					slug: org.slug,
				}
			} else if (owner) {
				target = {
					name: owner.user.username,
					avatar_url: owner.user.avatar_url,
					type: 'user',
					slug: owner.user.username,
				}
			}
		}

		return {
			...report,
			thread,
			reporter_user: cache.users.value.get(report.reporter) || ({} as User),
			project,
			user: report.item_type === 'user' ? cache.users.value.get(report.item_id) : undefined,
			version,
			target,
		}
	})
}

// Doesn't need to be in @modrinth/moderation because it is specific to the frontend.
export interface ModerationProject {
	project: any
	owner: TeamMember | null
	org: Organization | null
}

export async function enrichProjectBatch(projects: any[]): Promise<ModerationProject[]> {
	const teamIds = [...new Set(projects.map((p) => p.team_id).filter(Boolean))]
	const orgIds = [...new Set(projects.map((p) => p.organization).filter(Boolean))]

	const [teamsData, orgsData]: [TeamMember[][], Organization[]] = await Promise.all([
		teamIds.length > 0
			? fetchSegmented(teamIds, (ids) => `teams?ids=${asEncodedJsonArray(ids)}`)
			: Promise.resolve([]),
		orgIds.length > 0
			? fetchSegmented(orgIds, (ids) => `organizations?ids=${asEncodedJsonArray(ids)}`, {
					apiVersion: 3,
				})
			: Promise.resolve([]),
	])

	const cache = useModerationCache()

	teamsData.forEach((team) => {
		if (team.length > 0) cache.teams.value.set(team[0].team_id, team)
	})

	orgsData.forEach((org: Organization) => {
		cache.orgs.value.set(org.id, org)
	})

	return projects.map((project) => {
		let owner: TeamMember | null = null
		let org: Organization | null = null

		if (project.team_id) {
			const teamMembers = cache.teams.value.get(project.team_id)
			if (teamMembers) {
				owner = teamMembers.find((member) => member.role === 'Owner') || null
			}
		}

		if (project.organization) {
			org = cache.orgs.value.get(project.organization) || null
		}

		return {
			project,
			owner,
			org,
		} as ModerationProject
	})
}
