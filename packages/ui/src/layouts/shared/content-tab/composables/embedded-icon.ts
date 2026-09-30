import { ModrinthApiError } from '@modrinth/api-client'

const maxConcurrentRequests = 4
let activeRequests = 0
const pendingRequests: (() => void)[] = []

export async function fetchEmbeddedIcon(queryFn: () => Promise<Blob>): Promise<Blob | null> {
	if (activeRequests >= maxConcurrentRequests) {
		await new Promise<void>((resolve) => pendingRequests.push(resolve))
	} else {
		activeRequests += 1
	}

	try {
		return await queryFn()
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 404) return null
		throw error
	} finally {
		const next = pendingRequests.shift()
		if (next) next()
		else activeRequests -= 1
	}
}
