export function toError(error: unknown) {
	if (error instanceof Error) return error
	if (typeof error === 'string') return new Error(error)
	if (error && typeof error === 'object') {
		const record = error as Record<string, unknown>
		const message = record.message ?? record.error
		if (typeof message === 'string') return new Error(message)
		return new Error(JSON.stringify(error))
	}
	return new Error(String(error))
}
