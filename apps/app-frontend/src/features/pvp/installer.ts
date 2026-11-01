import { INFERNO_JARS, PVP_JVM_ARGS, type PvpMod, type PvpProfile } from './profiles'

/**
 * Pure orchestration of "build a PvP profile". All side effects go through `InstallerDeps`, so the
 * exact same logic runs against the real Tauri backend in the app and against mocks in tests.
 */

export type ModOutcome =
	| { slug: string; label: string; status: 'installed' }
	| { slug: string; label: string; status: 'skipped'; reason: string; optional: boolean }

export type InstallStage =
	| { kind: 'creating' }
	| { kind: 'mod'; index: number; total: number; label: string }
	| { kind: 'tuning' }
	| { kind: 'inferno' }
	| { kind: 'done' }

export interface InstallResult {
	instanceId: string
	outcomes: ModOutcome[]
	infernoInstalled: boolean
	infernoError?: string
	/** Whether the Java flags and memory limit were applied. Failure is non-fatal. */
	tuned: boolean
}

export interface InstallerDeps {
	/** Creates the (empty) Fabric instance and resolves once it is fully installed. Returns its id. */
	createInstance(args: {
		name: string
		gameVersion: string
		loader: 'fabric'
	}): Promise<string>
	/** Looks up a Modrinth project; must throw if it does not exist. Returns the canonical id. */
	resolveProjectId(slug: string): Promise<string>
	/**
	 * Installs the newest build of a project compatible with the version/loader, plus dependencies,
	 * and resolves when it is on disk. Must throw if no compatible build exists.
	 */
	installMod(args: {
		instanceId: string
		projectId: string
		gameVersion: string
		loader: 'fabric'
	}): Promise<void>
	/** Absolute path of a bundled resource, e.g. `inferno/inferno-client-1.21.11.jar`. */
	resolveBundledResource(relativePath: string): Promise<string>
	/** Applies the performance-tuned Java flags and heap size to the instance. */
	tuneInstance(args: { instanceId: string; jvmArgs: string[]; memoryMb: number }): Promise<void>
	/** Copies a local jar into the instance's mods folder. */
	addLocalMod(instanceId: string, absolutePath: string): Promise<void>
	/** Deletes a half-built instance after a hard failure. */
	removeInstance(instanceId: string): Promise<void>
}

function errorMessage(e: unknown): string {
	if (e instanceof Error) return e.message
	if (typeof e === 'string') return e
	try {
		const o = e as { message?: unknown; error?: unknown; description?: unknown }
		const m = o?.message ?? o?.error ?? o?.description
		if (typeof m === 'string') return m
		return JSON.stringify(e)
	} catch {
		return 'Unknown error'
	}
}

/** Picks a name that does not collide with an existing instance: "X", "X 2", "X 3", ... */
export function uniqueInstanceName(base: string, existing: Iterable<string>): string {
	const taken = new Set(Array.from(existing, (n) => n.trim().toLowerCase()))
	if (!taken.has(base.toLowerCase())) return base
	let n = 2
	while (taken.has(`${base} ${n}`.toLowerCase())) n++
	return `${base} ${n}`
}

export async function installPvpProfile(
	profile: PvpProfile,
	existingNames: Iterable<string>,
	deps: InstallerDeps,
	onStage: (stage: InstallStage) => void = () => {},
): Promise<InstallResult> {
	onStage({ kind: 'creating' })
	const name = uniqueInstanceName(profile.name, existingNames)

	const instanceId = await deps.createInstance({
		name,
		gameVersion: profile.gameVersion,
		loader: profile.loader,
	})

	try {
		const outcomes: ModOutcome[] = []
		const total = profile.mods.length

		// Sequential on purpose: parallel installs into one instance race on dependency resolution.
		for (let i = 0; i < total; i++) {
			const mod: PvpMod = profile.mods[i]
			onStage({ kind: 'mod', index: i + 1, total, label: mod.label })
			try {
				const projectId = await deps.resolveProjectId(mod.slug)
				await deps.installMod({
					instanceId,
					projectId,
					gameVersion: profile.gameVersion,
					loader: profile.loader,
				})
				outcomes.push({ slug: mod.slug, label: mod.label, status: 'installed' })
			} catch (e) {
				outcomes.push({
					slug: mod.slug,
					label: mod.label,
					status: 'skipped',
					reason: errorMessage(e),
					optional: !!mod.optional,
				})
			}
		}

		let tuned = false
		onStage({ kind: 'tuning' })
		try {
			await deps.tuneInstance({
				instanceId,
				jvmArgs: PVP_JVM_ARGS,
				memoryMb: profile.memoryMb,
			})
			tuned = true
		} catch {
			tuned = false
		}

		let infernoInstalled = false
		let infernoError: string | undefined
		if (profile.includeInferno) {
			onStage({ kind: 'inferno' })
			try {
				const jar = INFERNO_JARS[profile.gameVersion]
				if (!jar) throw new Error(`No Inferno build bundled for ${profile.gameVersion}`)
				const abs = await deps.resolveBundledResource(jar)
				await deps.addLocalMod(instanceId, abs)
				infernoInstalled = true
			} catch (e) {
				infernoError = errorMessage(e)
			}
		}

		// Inferno IS the point of these profiles. If it could not be installed, do not leave a
		// misleading "PvP profile" behind - remove it and surface the real error.
		if (profile.includeInferno && !infernoInstalled) {
			await deps.removeInstance(instanceId).catch(() => {})
			throw new Error(`Could not install Inferno Client: ${infernoError ?? 'unknown error'}`)
		}

		onStage({ kind: 'done' })
		return { instanceId, outcomes, infernoInstalled, infernoError, tuned }
	} catch (e) {
		// Unexpected failure after the instance exists: clean up so retries start fresh.
		if (!(e instanceof Error && e.message.startsWith('Could not install Inferno Client'))) {
			await deps.removeInstance(instanceId).catch(() => {})
		}
		throw e
	}
}
