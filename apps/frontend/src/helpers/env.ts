type SecretsStoreBinding = { get: () => Promise<string> }

// Cloudflare exposes plain vars and Secrets Store bindings on the Workers env. Every
// other runtime (the Docker image, local dev) only has process.env.
async function getWorkersEnv(): Promise<Record<string, unknown> | undefined> {
	try {
		const mod = 'cloudflare:workers'
		const { env } = await import(/* @vite-ignore */ mod)
		return env as Record<string, unknown>
	} catch {
		return undefined
	}
}

export async function readEnv(name: string): Promise<string | undefined> {
	const binding = (await getWorkersEnv())?.[name]

	if (typeof binding === 'string') return binding
	if (binding) return await (binding as SecretsStoreBinding).get()

	return globalThis.process?.env?.[name]
}
