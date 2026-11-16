export const SHARED_INSTANCE_CONFIG_FILE_TYPES = new Set(['config', 'configs'])

export function getRequestErrorMessage(error: unknown, fallback: string): string {
	if (typeof error === 'string') return error
	if (!error || typeof error !== 'object') return fallback

	const requestError = error as { message?: string; data?: { description?: string } }
	return requestError.data?.description ?? requestError.message ?? fallback
}

export function sharedInstanceInspectorUrl(ownerId: string, instanceId: string, version?: number) {
	const params = new URLSearchParams({ instance: instanceId })
	if (version !== undefined) params.set('version', String(version))
	return `${window.location.origin}/admin/shared-instances/${encodeURIComponent(ownerId)}?${params.toString()}`
}

export function formatSharedInstanceLoader(version: { loader?: string; loader_version?: string }) {
	const loader = version.loader
		? version.loader.charAt(0).toUpperCase() + version.loader.slice(1)
		: 'Vanilla'
	return version.loader_version ? `${loader} ${version.loader_version}` : loader
}
