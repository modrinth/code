const SERVER_RESUBSCRIBE_WINDOW_MS = 30 * 24 * 60 * 60 * 1000

export function isWithinServerResubscribeWindow(
	cancellationDate: string | number | Date | null | undefined,
): boolean {
	if (cancellationDate == null) return false
	const cancellationTime = new Date(cancellationDate).getTime()
	return (
		Number.isFinite(cancellationTime) &&
		Date.now() <= cancellationTime + SERVER_RESUBSCRIBE_WINDOW_MS
	)
}
