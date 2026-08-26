import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'
import { validateProjectFields } from '@modrinth/moderation'

export type ValidationFilterRequest = Omit<
	Labrinth.Moderation.Internal.ProjectsRequest,
	'count' | 'offset' | 'project_type'
>

export interface ModerationQueueFetchOptions {
	client: AbstractModrinthClient
	request: ValidationFilterRequest
	signal: AbortSignal
	log: (message: string) => void
}

interface ValidationFilterScanOptions {
	client: AbstractModrinthClient
	request: ValidationFilterRequest
	includeWarnings: boolean
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

export async function fetchAllModerationQueueProjects(
	options: ModerationQueueFetchOptions,
): Promise<Labrinth.Moderation.Internal.ProjectsResponse> {
	const pacedFetcher = createPacedFetcher(options.signal, options.log)

	try {
		options.log('Starting moderation queue fetch')
		const response = await fetchQueueProjects(options, pacedFetcher.fetch)
		options.log(`Queue fetch complete: ${response.total} projects fetched`)
		return response
	} catch (error) {
		if (options.signal.aborted) {
			options.log('Queue fetch cancelled')
		} else {
			console.error(
				`[moderation-project-ids-filter] Queue fetch failed during ${pacedFetcher.getCurrentStage()}`,
				error,
			)
		}
		throw error
	}
}

export async function scanProjectsWithValidationIssues({
	client,
	request,
	includeWarnings,
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
			const projects = await pacedFetcher.fetch(
				`Fetching V3 project batch ${batchIndex + 1}/${projectBatchCount}`,
				() => client.labrinth.projects_v3.getMultiple(projectIds),
			)
			const projectsById = new Map(projects.map((project) => [project.id, project]))
			const missingProjectIds = projectIds.filter((projectId) => !projectsById.has(projectId))

			if (missingProjectIds.length > 0) {
				throw new Error(`V3 projects response omitted ${missingProjectIds.length} queued projects`)
			}

			for (const projectId of projectIds) {
				const project = projectsById.get(projectId)
				if (!project) {
					throw new Error(`V3 projects response omitted queued project ${projectId}`)
				}
				const validation = validateProjectFields(project)
				if (includeWarnings ? validation.failures.length > 0 : !validation.valid) {
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
