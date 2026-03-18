import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'

type Notification = Labrinth.Notifications.v2.Notification

export type PlatformNotification = Notification & {
	extra_data?: Record<string, unknown>
	grouped_notifs?: PlatformNotification[]
}

async function safeBulkFetch<T>(fn: () => Promise<T[]>): Promise<T[]> {
	try {
		const res = await fn()
		return Array.isArray(res) ? res : []
	} catch {
		return []
	}
}

export async function fetchExtraNotificationData(
	client: AbstractModrinthClient,
	notifications: PlatformNotification[],
): Promise<PlatformNotification[]> {
	const bulk = {
		projects: [] as string[],
		reports: [] as string[],
		threads: [] as string[],
		users: [] as string[],
		versions: [] as string[],
		organizations: [] as string[],
	}

	for (const notification of notifications) {
		if (notification.body) {
			if (notification.body.project_id) bulk.projects.push(notification.body.project_id)
			if (notification.body.version_id) bulk.versions.push(notification.body.version_id)
			if (notification.body.report_id) bulk.reports.push(notification.body.report_id)
			if (notification.body.thread_id) bulk.threads.push(notification.body.thread_id)
			if (notification.body.invited_by) bulk.users.push(notification.body.invited_by)
			if (notification.body.organization_id)
				bulk.organizations.push(notification.body.organization_id)
		}
	}

	const reports = (
		await safeBulkFetch(() =>
			bulk.reports.length > 0
				? client.labrinth.reports_v3.getMultiple([...new Set(bulk.reports)])
				: Promise.resolve([]),
		)
	).filter(Boolean)

	for (const r of reports) {
		if (!r?.item_type) continue
		if (r.item_type === 'project') bulk.projects.push(r.item_id)
		else if (r.item_type === 'user') bulk.users.push(r.item_id)
		else if (r.item_type === 'version') bulk.versions.push(r.item_id)
	}

	const versions = (
		await safeBulkFetch(() =>
			bulk.versions.length > 0
				? client.labrinth.versions_v2.getVersions([...new Set(bulk.versions)])
				: Promise.resolve([]),
		)
	).filter(Boolean)

	for (const v of versions) bulk.projects.push(v.project_id)

	const [projects, threads, users, organizations] = await Promise.all([
		safeBulkFetch(() =>
			bulk.projects.length > 0
				? client.labrinth.projects_v2.getMultiple([...new Set(bulk.projects)])
				: Promise.resolve([]),
		),
		safeBulkFetch(() =>
			bulk.threads.length > 0
				? client.labrinth.threads_v3.getMultiple([...new Set(bulk.threads)])
				: Promise.resolve([]),
		),
		safeBulkFetch(() =>
			bulk.users.length > 0
				? client.labrinth.users_v2.getMultiple([...new Set(bulk.users)])
				: Promise.resolve([]),
		),
		safeBulkFetch(() =>
			bulk.organizations.length > 0
				? client.labrinth.organizations_v3.getMultiple([...new Set(bulk.organizations)])
				: Promise.resolve([]),
		),
	])

	type Report = Labrinth.Reports.v3.Report
	type Version = Labrinth.Versions.v2.Version

	for (const n of notifications) {
		n.extra_data = {}
		if (n.body) {
			if (n.body.project_id)
				n.extra_data.project = projects.find((x) => x.id === n.body!.project_id)
			if (n.body.organization_id)
				n.extra_data.organization = organizations.find((x) => x.id === n.body!.organization_id)
			if (n.body.report_id) {
				n.extra_data.report = reports.find((x) => x.id === n.body!.report_id)
				const t = (n.extra_data.report as Report | undefined)?.item_type
				if (t === 'project')
					n.extra_data.project = projects.find(
						(x) => x.id === (n.extra_data?.report as Report | undefined)?.item_id,
					)
				else if (t === 'user')
					n.extra_data.user = users.find(
						(x) => x.id === (n.extra_data?.report as Report | undefined)?.item_id,
					)
				else if (t === 'version') {
					n.extra_data.version = versions.find(
						(x) => x.id === (n.extra_data?.report as Report | undefined)?.item_id,
					)
					n.extra_data.project = projects.find(
						(x) => x.id === (n.extra_data?.version as Version | undefined)?.project_id,
					)
				}
			}
			if (n.body.thread_id) n.extra_data.thread = threads.find((x) => x.id === n.body!.thread_id)
			if (n.body.invited_by)
				n.extra_data.invited_by = users.find((x) => x.id === n.body!.invited_by)
			if (n.body.version_id)
				n.extra_data.version = versions.find((x) => x.id === n.body!.version_id)
		}
	}
	return notifications
}

export function groupNotifications(notifications: PlatformNotification[]): PlatformNotification[] {
	const grouped: PlatformNotification[] = []
	for (let i = 0; i < notifications.length; i++) {
		const current = notifications[i]
		const next = notifications[i + 1]
		if (current.body && i < notifications.length - 1 && isSimilar(current, next)) {
			const groupedNotif = { ...current, grouped_notifs: [next] }
			let j = i + 2
			while (j < notifications.length && isSimilar(current, notifications[j])) {
				groupedNotif.grouped_notifs.push(notifications[j])
				j++
			}
			grouped.push(groupedNotif)
			i = j - 1
		} else {
			grouped.push(current)
		}
	}
	return grouped
}

function isSimilar(a: PlatformNotification, b: PlatformNotification | undefined): boolean {
	return !!a?.body?.project_id && a.body!.project_id === b?.body?.project_id
}

export async function markAsRead(
	client: AbstractModrinthClient,
	ids: string[],
): Promise<(notifications: PlatformNotification[]) => PlatformNotification[]> {
	await client.labrinth.notifications_v2.markMultipleAsRead(ids)
	return (notifications: PlatformNotification[]) => {
		const newNotifs = notifications ?? []
		newNotifs.forEach((n) => {
			if (ids.includes(n.id)) n.read = true
		})
		return newNotifs
	}
}
