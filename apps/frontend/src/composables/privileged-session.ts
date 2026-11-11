// must match labrinth PRIVILEGED_TOKEN_INVALID_IN_SEC, update both together
export const PRIVILEGED_SESSION_MS = 15 * 60 * 1000

export function isIssuedWithinPrivilegedWindow(createdAt: number, now = Date.now()): boolean {
	return now - createdAt < PRIVILEGED_SESSION_MS
}

export function sessionCreatedAt(created: string | number): number | null {
	const at = typeof created === 'number' ? created : Date.parse(created)
	if (!Number.isFinite(at)) {
		return null
	}
	return at
}

function errorCode(value: unknown): string | undefined {
	if (!value || typeof value !== 'object') {
		return undefined
	}
	const error = (value as { error?: unknown }).error
	if (typeof error !== 'string') {
		return undefined
	}
	return error
}

export function isPrivilegedTokenExpired(error: unknown): boolean {
	if (!error || typeof error !== 'object') {
		return false
	}
	const record = error as {
		data?: unknown
		responseData?: unknown
		v1Error?: unknown
	}
	return [record, record.data, record.responseData, record.v1Error].some(
		(source) => errorCode(source) === 'privileged_token_expired',
	)
}
