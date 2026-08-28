import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'
import { type ProjectValidationContext, validateProject } from '@modrinth/moderation'

export type ValidationFilterRequest = Omit<
	Labrinth.Moderation.Internal.ProjectsRequest,
	'count' | 'offset' | 'project_type'
>

interface ModerationQueueFetchOptions {
	client: AbstractModrinthClient
	request: ValidationFilterRequest
	signal: AbortSignal
	log: (message: string) => void
}

interface ValidationFilterScanOptions {
	client: AbstractModrinthClient
	request: ValidationFilterRequest
	includeWarnings: boolean
	tags: ProjectValidationContext['tags']
	signal: AbortSignal
	log: (message: string) => void
}

const REQUEST_DELAY_MS = 500
const PROJECT_BATCH_SIZE = 100
const QUEUE_PAGE_SIZE = 200

function createPacedFetcher(signal: AbortSignal, log: (message: string) => void) {
	let requestCount = 0
	let currentStage = 'starting request'

	return {
		async fetch<T>(label: string, fetcher: () => Promise<T>): Promise<T> {
			signal.throwIfAborted()
			if (requestCount > 0) {
				log(`Waiting ${REQUEST_DELAY_MS}ms before next request`)
				await waitForNextRequest(signal)
			}

			currentStage = label
			log(label)
			requestCount++
			const result = await fetcher()
			signal.throwIfAborted()
			return result
		},
		getCurrentStage() {
			return currentStage
		},
	}
}

function waitForNextRequest(signal: AbortSignal): Promise<void> {
	return new Promise((resolve, reject) => {
		if (signal.aborted) {
			reject(signal.reason ?? new Error('Moderation queue fetch cancelled'))
			return
		}

		const timeout = setTimeout(() => {
			signal.removeEventListener('abort', onAbort)
			resolve()
		}, REQUEST_DELAY_MS)

		function onAbort() {
			clearTimeout(timeout)
			reject(signal.reason ?? new Error('Moderation queue fetch cancelled'))
		}

		signal.addEventListener('abort', onAbort, { once: true })
	})
}

async function fetchQueueProjects(
	{ client, request, log }: ModerationQueueFetchOptions,
	fetchWithDelay: ReturnType<typeof createPacedFetcher>['fetch'],
): Promise<Labrinth.Moderation.Internal.ProjectsResponse> {
	const firstPage = await fetchWithDelay('Fetching queue page 1', () =>
		client.labrinth.moderation_internal.getProjects({
			...request,
			count: QUEUE_PAGE_SIZE,
			offset: 0,
		}),
	)
	const projects = [...firstPage.projects]
	const pageCount = Math.ceil(firstPage.total / QUEUE_PAGE_SIZE)

	log(`Found ${firstPage.total} queue projects`)
	for (let page = 1; page < pageCount; page++) {
		const response = await fetchWithDelay(`Fetching queue page ${page + 1}/${pageCount}`, () =>
			client.labrinth.moderation_internal.getProjects({
				...request,
				count: QUEUE_PAGE_SIZE,
				offset: page * QUEUE_PAGE_SIZE,
			}),
		)
		projects.push(...response.projects)
	}

	return {
		total: projects.length,
		projects,
	}
}

export async function scanProjectsWithValidationIssues({
	client,
	request,
	includeWarnings,
	tags,
	signal,
	log,
}: ValidationFilterScanOptions): Promise<Labrinth.Moderation.Internal.ProjectsResponse> {
	const pacedFetcher = createPacedFetcher(signal, log)

	try {
		log('Starting validation scan')
		const queueResponse = await fetchQueueProjects(
			{ client, request, signal, log },
			pacedFetcher.fetch,
		)
		const queueProjects = queueResponse.projects
		log(`Found ${queueProjects.length} projects to scan`)

		const matchingProjectIds = new Set<string>()
		let validatedProjectCount = 0
		const projectBatchCount = Math.ceil(queueProjects.length / PROJECT_BATCH_SIZE)

		for (let batchIndex = 0; batchIndex < projectBatchCount; batchIndex++) {
			const projectIds = queueProjects
				.slice(batchIndex * PROJECT_BATCH_SIZE, (batchIndex + 1) * PROJECT_BATCH_SIZE)
				.map((project) => project.id)
			const projectsV3 = await pacedFetcher.fetch(
				`Fetching V3 project batch ${batchIndex + 1}/${projectBatchCount}`,
				() => client.labrinth.projects_v3.getMultiple(projectIds),
			)
			const projectsV2 = await pacedFetcher.fetch(
				`Fetching V2 project batch ${batchIndex + 1}/${projectBatchCount}`,
				() => client.labrinth.projects_v2.getMultiple(projectIds),
			)
			const projectsV3ById = new Map(projectsV3.map((project) => [project.id, project]))
			const projectsV2ById = new Map(projectsV2.map((project) => [project.id, project]))
			const missingProjectIds = projectIds.filter(
				(projectId) => !projectsV3ById.has(projectId) || !projectsV2ById.has(projectId),
			)

			if (missingProjectIds.length > 0) {
				throw new Error(`Project responses omitted ${missingProjectIds.length} queued projects`)
			}

			const versionIds = [...new Set(projectsV3.flatMap((project) => project.versions))]
			const versions: Labrinth.Versions.v3.Version[] = []
			const versionBatchCount = Math.ceil(versionIds.length / PROJECT_BATCH_SIZE)
			for (let versionBatchIndex = 0; versionBatchIndex < versionBatchCount; versionBatchIndex++) {
				const batchVersionIds = versionIds.slice(
					versionBatchIndex * PROJECT_BATCH_SIZE,
					(versionBatchIndex + 1) * PROJECT_BATCH_SIZE,
				)
				versions.push(
					...(await pacedFetcher.fetch(
						`Fetching version batch ${versionBatchIndex + 1}/${versionBatchCount}`,
						() => client.labrinth.versions_v3.getVersions(batchVersionIds),
					)),
				)
			}
			const versionsById = new Map(versions.map((version) => [version.id, version]))
			const missingVersionIds = versionIds.filter((versionId) => !versionsById.has(versionId))
			if (missingVersionIds.length > 0) {
				throw new Error(`Version responses omitted ${missingVersionIds.length} project versions`)
			}

			for (const projectId of projectIds) {
				const projectV3 = projectsV3ById.get(projectId)
				const rawProjectV2 = projectsV2ById.get(projectId)
				if (!projectV3 || !rawProjectV2) {
					throw new Error(`Project responses omitted queued project ${projectId}`)
				}
				const project = {
					...rawProjectV2,
					actualProjectType: rawProjectV2.project_type,
				}
				const projectVersions = projectV3.versions.flatMap((versionId) => {
					const version = versionsById.get(versionId)
					return version ? [version] : []
				})
				const validation = validateProject({
					project,
					projectV3,
					versions: projectVersions,
					tags,
				})
				if (
					validation.requiredNags.length > 0 ||
					(includeWarnings && validation.warningNags.length > 0)
				) {
					matchingProjectIds.add(projectId)
				}
			}

			validatedProjectCount += projectIds.length
			log(
				`Validated ${validatedProjectCount}/${queueProjects.length} projects; ${matchingProjectIds.size} matched`,
			)
		}

		const projects = queueProjects.filter((project) => matchingProjectIds.has(project.id))
		log(`Matching project IDs: ${projects.map((project) => project.id).join(', ') || 'none'}`)
		log(`Scan complete: ${projects.length}/${queueProjects.length} projects matched`)

		return {
			total: projects.length,
			projects,
		}
	} catch (error) {
		if (signal.aborted) {
			log('Scan cancelled')
		} else {
			console.error(
				`[moderation-validation-filter] Scan failed during ${pacedFetcher.getCurrentStage()}`,
				error,
			)
		}
		throw error
	}
}
