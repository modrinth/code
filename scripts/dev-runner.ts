#!/usr/bin/env node

import * as NodeChildProcess from 'node:child_process'
import * as NodeCrypto from 'node:crypto'
import * as NodeFS from 'node:fs'
import * as NodeNet from 'node:net'
import * as NodePath from 'node:path'
import * as NodeReadline from 'node:readline'
import * as NodeURL from 'node:url'
import * as NodeUtil from 'node:util'
import { normalizeScenarioDatabase, runningAppCommands } from './scenario-state.ts'

const BASE_PORTS = {
	app: 1420,
	backend: 8787,
	accountWeb: 3100,
	core: 16662,
} as const
const MAX_HASH_OFFSET = 3000
const MAX_PORT = 65_535
const RESTART_FAILURE_LIMIT = 3
const RESTART_FAILURE_WINDOW_MS = 10_000
const PORT_PROBE_HOSTS = ['127.0.0.1', '::1'] as const
const FETCH_BAD_PORTS = new Set([
	0, 1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95, 101, 102,
	103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161, 179, 389, 427, 465,
	512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563, 587, 601, 636, 989, 990, 993,
	995, 1719, 1720, 1723, 2049, 3659, 4045, 4190, 5060, 5061, 6000, 6566, 6665, 6666, 6667, 6668,
	6669, 6679, 6697, 10080,
])

export const DEV_MODES = ['dev', 'dev:app', 'dev:backend', 'dev:core'] as const
export type DevMode = (typeof DEV_MODES)[number]
export type DevPorts = {
	readonly app: number
	readonly backend: number
	readonly accountWeb: number
	readonly core: number
}

type PortName = keyof DevPorts
type PortAvailabilityCheck = (port: number, hosts: ReadonlyArray<string>) => Promise<boolean>
export type WorktreePaths = {
	readonly coreData: string
	readonly data: string
	readonly primary: string
	readonly runtime: string
	readonly scenariosData: string
	readonly worktree: string
}
type RunnerInput = {
	readonly dryRun: boolean
	readonly mode: DevMode
	readonly scenarios: ReadonlyArray<number>
}
export type ProcessSpec = {
	readonly executable?: string
	readonly args: ReadonlyArray<string>
	readonly cwd: string
	readonly env: NodeJS.ProcessEnv
	readonly label: string
}
type RunningProcess = {
	readonly child: NodeChildProcess.ChildProcess
	readonly done: Promise<number>
	readonly label: string
	readonly recentOutput: string[]
}
type LogLevel = 'error' | 'info' | 'warning'

export class DevRunnerError extends Error {
	constructor(message: string) {
		super(message)
		this.name = 'DevRunnerError'
	}
}

export function scenarioUsername(scenario: number): string {
	return `scenario_${scenario}`
}

export function isBrowserAllowedPort(port: number): boolean {
	return !FETCH_BAD_PORTS.has(port)
}

export function portsForOffset(offset: number): DevPorts {
	return {
		app: BASE_PORTS.app + offset,
		backend: BASE_PORTS.backend + offset,
		accountWeb: BASE_PORTS.accountWeb + offset,
		core: BASE_PORTS.core + offset,
	}
}

export function resolveStartOffset(input: {
	readonly devInstance?: string
	readonly explicitOffset?: string
	readonly primaryPath: string
	readonly worktreePath: string
}): { readonly offset: number; readonly source: string } {
	const explicit = input.explicitOffset?.trim()
	if (explicit) {
		if (!/^\d+$/.test(explicit)) {
			throw new DevRunnerError(
				`AMBERITE_PORT_OFFSET must be a non-negative integer; received ${explicit}.`,
			)
		}
		return { offset: Number(explicit), source: `AMBERITE_PORT_OFFSET=${explicit}` }
	}

	const instance = input.devInstance?.trim()
	if (instance) {
		if (/^\d+$/.test(instance)) {
			return { offset: Number(instance), source: `numeric AMBERITE_DEV_INSTANCE=${instance}` }
		}
		return {
			offset: (stableHash(instance) % MAX_HASH_OFFSET) + 1,
			source: `hashed AMBERITE_DEV_INSTANCE=${instance}`,
		}
	}

	if (samePath(input.primaryPath, input.worktreePath)) {
		return { offset: 0, source: 'primary checkout' }
	}

	return {
		offset: (stableHash(input.worktreePath) % MAX_HASH_OFFSET) + 1,
		source: `worktree ${input.worktreePath}`,
	}
}

export async function findFirstAvailableOffset(input: {
	readonly checkPort?: PortAvailabilityCheck
	readonly mode: DevMode
	readonly startOffset: number
}): Promise<number> {
	const requiredPorts = requiredPortNames(input.mode)
	const checkPort = input.checkPort ?? portIsAvailable

	for (let offset = input.startOffset; offset <= MAX_PORT; offset += 1) {
		const ports = portsForOffset(offset)
		if (requiredPorts.some((name) => ports[name] > MAX_PORT)) break
		if (requiredPorts.some((name) => !isBrowserAllowedPort(ports[name]))) continue

		const available = await Promise.all(
			requiredPorts.map((name) => checkPort(ports[name], PORT_PROBE_HOSTS)),
		)
		if (available.every(Boolean)) return offset
	}

	throw new DevRunnerError(`No development ports are available from offset ${input.startOffset}.`)
}

export function createRuntimeEnvironment(input: {
	readonly baseEnv: NodeJS.ProcessEnv
	readonly paths: WorktreePaths
	readonly ports: DevPorts
}): NodeJS.ProcessEnv {
	const env: NodeJS.ProcessEnv = { ...input.baseEnv }
	delete env.HOST
	delete env.PORT
	env.ACCOUNT_API_URL = `http://127.0.0.1:${input.ports.backend}`
	env.VITE_ACCOUNT_API_URL = env.ACCOUNT_API_URL
	env.ACCOUNT_WEB_URL = `http://127.0.0.1:${input.ports.accountWeb}`
	return {
		...env,
		AMBERITE_DATA_DIR: input.paths.data,
		AMBERITE_DEV_MODE: 'true',
		AMBERITE_LOCAL_CORE_DATA_DIR: input.paths.coreData,
	}
}

export function processLabelsForMode(mode: DevMode): ReadonlyArray<string> {
	switch (mode) {
		case 'dev':
			return ['backend', 'account-web', 'core', 'app-frontend']
		case 'dev:backend':
			return ['backend', 'account-web']
		case 'dev:app':
			return ['app-frontend']
		case 'dev:core':
			return ['core']
	}
}

async function main(): Promise<void> {
	const paths = resolveWorktreePaths()
	const input = parseInput(
		process.argv.slice(2),
		readDefaultScenarios(NodePath.join(paths.worktree, 'dev.json')),
	)
	const { offset: startOffset, source } = resolveStartOffset({
		devInstance: process.env.AMBERITE_DEV_INSTANCE,
		explicitOffset: process.env.AMBERITE_PORT_OFFSET,
		primaryPath: paths.primary,
		worktreePath: paths.worktree,
	})
	const selectedOffset = await findFirstAvailableOffset({
		mode: input.mode,
		startOffset,
	})
	const ports = portsForOffset(selectedOffset)
	const branch = currentBranch(paths.worktree)
	const sharedEnv = {
		...readEnv(NodePath.join(paths.worktree, 'packages', 'app-lib', '.env.prod')),
		...readEnv(NodePath.join(paths.worktree, 'packages', 'app-lib', '.env')),
		...readEnv(NodePath.join(paths.primary, '.env.local')),
		...readEnv(NodePath.join(paths.primary, 'apps', 'core', '.env.local')),
		...readEnv(NodePath.join(paths.worktree, '.env.local')),
		...process.env,
	}
	const env = createRuntimeEnvironment({ baseEnv: sharedEnv, paths, ports })
	if (!input.dryRun) {
		const secretPath = NodePath.join(paths.data, 'backend', 'dev-secret')
		NodeFS.mkdirSync(NodePath.dirname(secretPath), { recursive: true })
		if (!NodeFS.existsSync(secretPath))
			NodeFS.writeFileSync(secretPath, NodeCrypto.randomBytes(32).toString('hex'), { mode: 0o600 })
		env.AMBERITE_LOCAL_DEV_SECRET = NodeFS.readFileSync(secretPath, 'utf8').trim()
		if (processLabelsForMode(input.mode).includes('backend')) {
			prepareLocalBackend(paths, env)
		}
	}
	const specs = createProcessSpecs({
		branch,
		env,
		mode: input.mode,
		paths,
		ports,
		scenarios: input.scenarios,
	})
	printPlan({
		branch,
		mode: input.mode,
		paths,
		ports,
		scenarios: input.scenarios,
		source,
		specs,
	})
	if (input.dryRun) return

	ensureDataLayout(paths, input.scenarios)
	writeRuntimeFile({
		branch,
		mode: input.mode,
		paths,
		ports,
		scenarios: input.scenarios,
		source,
	})
	const specsByLabel = new Map(specs.map((spec) => [spec.label, spec]))
	const processes = new Map<string, RunningProcess>()
	const restartFailures = new Map<string, number[]>()
	let stopping = false
	let finish: (() => void) | undefined
	const finished = new Promise<void>((resolve) => {
		finish = resolve
	})

	const stop = async (exitCode: number) => {
		if (stopping) return
		stopping = true
		commands.close()
		await Promise.all([...processes.values()].map(stopProcess))
		processes.clear()
		process.exitCode = exitCode
		finish?.()
	}
	const start = (spec: ProcessSpec) => {
		if (spec.label.startsWith('app:')) {
			const dataDir = spec.env.THESEUS_CONFIG_DIR
			if (!dataDir) throw new DevRunnerError('App scenario is missing its data directory.')
			normalizeScenarioDatabase(paths.worktree, dataDir, runningAppCommands())
		}
		const running = spawnProcess(paths.worktree, spec)
		processes.set(spec.label, running)
		void running.done.then(
			(code) => handleProcessExit(spec, running, code),
			(error: unknown) => {
				runnerLog(
					'error',
					`${spec.label} failed: ${error instanceof Error ? error.message : String(error)}`,
				)
				void handleProcessExit(spec, running, 1)
			},
		)
		return running
	}
	const restart = async (label: string) => {
		const spec = specsByLabel.get(label)
		if (!spec || (label !== 'core' && !label.startsWith('app:'))) {
			runnerLog('warning', `Unknown restart target ${label}. Use rs <scenario> or rs core.`)
			return
		}
		const current = processes.get(label)
		if (current) {
			processes.delete(label)
			await stopProcess(current)
		}
		restartFailures.delete(label)
		runnerLog('info', `Restarting ${label}...`)
		start(spec)
	}
	const handleProcessExit = async (spec: ProcessSpec, running: RunningProcess, code: number) => {
		if (processes.get(spec.label) !== running) return
		processes.delete(spec.label)
		if (stopping) return

		if (spec.label === 'core' || (spec.label.startsWith('app:') && code !== 0)) {
			const now = Date.now()
			const failures = [...(restartFailures.get(spec.label) ?? []), now].filter(
				(timestamp) => now - timestamp < RESTART_FAILURE_WINDOW_MS,
			)
			restartFailures.set(spec.label, failures)
			if (failures.length >= RESTART_FAILURE_LIMIT) {
				const command = spec.label === 'core' ? 'rs core' : `rs ${spec.label.slice(4)}`
				runnerLog(
					'error',
					`${spec.label} failed ${RESTART_FAILURE_LIMIT} times in 10 seconds; automatic restarts paused. Use ${command} after fixing it.`,
				)
				return
			}
			if (appExecutableIsLocked(running)) {
				runnerLog(
					'warning',
					`${spec.label} is waiting for the existing App process to release theseus_gui.exe...`,
				)
				for (;;) {
					if (stopping || processes.has(spec.label) || !isWindowsProcessRunning('theseus_gui.exe'))
						break
					await new Promise((resolve) => setTimeout(resolve, 500))
				}
				if (!stopping && !processes.has(spec.label)) start(spec)
				return
			}
			runnerLog('warning', `${spec.label} exited with code ${code}; restarting...`)
			await new Promise((resolve) => setTimeout(resolve, 750))
			if (!stopping && !processes.has(spec.label)) start(spec)
			return
		}
		if (spec.label.startsWith('app:')) {
			runnerLog('info', `${spec.label} closed. Use rs ${spec.label.slice(4)} to start it again.`)
			return
		}

		runnerLog('error', `${spec.label} exited with code ${code}.`)
		await stop(code || 1)
	}
	const commands = createCommandInput({
		getCore: () => processes.get('core'),
		restart: (target) => restart(target === 'core' ? target : `app:${target}`),
		stop: () => stop(0),
	})

	process.once('SIGINT', () => void stop(130))
	process.once('SIGTERM', () => void stop(143))

	try {
		for (const spec of specs) start(spec)
		await finished
	} finally {
		await stop(process.exitCode || 1)
	}
}

export function parseInput(
	args: ReadonlyArray<string>,
	defaultScenarios: ReadonlyArray<number> = [1],
): RunnerInput {
	const modeValue = args.find((arg) => !arg.startsWith('-')) ?? 'dev'
	if (!DEV_MODES.includes(modeValue as DevMode)) {
		throw new DevRunnerError(
			`Unknown development mode ${modeValue}. Expected ${DEV_MODES.join(', ')}.`,
		)
	}

	const scenarioArgs = args.filter((arg) => arg !== modeValue && arg !== '--dry-run')
	const startsApps = modeValue === 'dev' || modeValue === 'dev:app'
	if (!startsApps && scenarioArgs.length > 0) {
		throw new DevRunnerError(`${modeValue} does not start App scenarios.`)
	}
	const scenarios = startsApps
		? normalizeScenarios(scenarioArgs.length > 0 ? scenarioArgs : defaultScenarios)
		: []

	return { dryRun: args.includes('--dry-run'), mode: modeValue as DevMode, scenarios }
}

export function readDefaultScenarios(path: string): ReadonlyArray<number> {
	if (!NodeFS.existsSync(path)) return [1]
	let value: unknown
	try {
		value = JSON.parse(NodeFS.readFileSync(path, 'utf8'))
	} catch (error) {
		throw new DevRunnerError(`Could not read ${path}: ${String(error)}`)
	}
	if (!isRecord(value) || !Array.isArray(value.defaultScenarios)) {
		throw new DevRunnerError(`${path} must contain a defaultScenarios array.`)
	}
	return normalizeScenarios(value.defaultScenarios)
}

function normalizeScenarios(values: ReadonlyArray<unknown>): ReadonlyArray<number> {
	const scenarios = values.map((value) => {
		const scenario = typeof value === 'number' ? value : Number(value)
		if (!Number.isSafeInteger(scenario) || scenario < 1 || scenario > 999_999_999_999) {
			throw new DevRunnerError(`Scenario ${String(value)} must be a positive whole number.`)
		}
		return scenario
	})
	if (scenarios.length === 0) throw new DevRunnerError('At least one App scenario is required.')
	return [...new Set(scenarios)]
}

function resolveWorktreePaths(): WorktreePaths {
	const worktree = git(['rev-parse', '--show-toplevel'], process.cwd())
	const worktreeList = git(['worktree', 'list', '--porcelain'], worktree)
	const primary =
		worktreeList
			.split(/\r?\n/)
			.find((line) => line.startsWith('worktree '))
			?.slice('worktree '.length) ?? worktree
	const data = NodePath.join(worktree, '.data')

	return {
		coreData: NodePath.join(data, 'core'),
		data,
		primary: NodePath.resolve(primary),
		runtime: NodePath.join(data, 'runtime.json'),
		scenariosData: NodePath.join(data, 'scenarios'),
		worktree: NodePath.resolve(worktree),
	}
}

function ensureDataLayout(paths: WorktreePaths, scenarios: ReadonlyArray<number>): void {
	NodeFS.mkdirSync(paths.data, { recursive: true })
	NodeFS.mkdirSync(paths.coreData, { recursive: true })
	for (const scenario of new Set([1, 2, 3, 4, ...scenarios])) {
		NodeFS.mkdirSync(scenarioDataPath(paths, scenario), { recursive: true })
	}
}

export function createProcessSpecs(input: {
	readonly branch: string
	readonly env: NodeJS.ProcessEnv
	readonly mode: DevMode
	readonly paths: WorktreePaths
	readonly ports: DevPorts
	readonly scenarios: ReadonlyArray<number>
}): ReadonlyArray<ProcessSpec> {
	const labels = processLabelsForMode(input.mode)
	const tauri = JSON.parse(
		NodeFS.readFileSync(
			NodePath.join(input.paths.worktree, 'apps', 'app', 'tauri.conf.json'),
			'utf8',
		),
	) as {
		readonly app: {
			readonly security: {
				readonly csp: Record<string, string>
				readonly capabilities: readonly unknown[]
			}
		}
	}
	const connectSrc = tauri.app.security.csp['connect-src']
	const backendUrl = `http://127.0.0.1:${input.ports.backend}`
	const tauriOverride = {
		app: {
			security: {
				capabilities: [
					...tauri.app.security.capabilities,
					{
						identifier: 'local-account-backend',
						windows: ['main'],
						permissions: [
							{
								identifier: 'http:default',
								// Paired Cores report their own address, so it is not known up front.
								allow: [
									{ url: `${backendUrl}/*` },
									{ url: 'http://*:*/*' },
									{ url: 'https://*:*/*' },
								],
							},
						],
					},
				],
				csp: {
					'img-src': `${tauri.app.security.csp['img-src']} ${backendUrl}`,
					'connect-src': `${connectSrc} http://localhost:${input.ports.app} ws://localhost:${input.ports.app} ${backendUrl} ${backendUrl.replace('http', 'ws')} http: https: ws: wss:`,
				},
			},
		},
		build: {
			beforeDevCommand: '',
			devUrl: `http://localhost:${input.ports.app}`,
		},
	}
	const coreEnv = {
		...input.env,
		ALLOWED_ORIGIN: `http://localhost:${input.ports.app}`,
		AMBERITE_BIND_HOST: '127.0.0.1',
		CORE_DATA_DIR: input.paths.coreData,
		PORT: String(input.ports.core),
	}
	const specs: Record<string, ProcessSpec> = {
		backend: {
			args: [
				'exec',
				'wrangler',
				'dev',
				'--local',
				'--ip',
				'127.0.0.1',
				'--port',
				String(input.ports.backend),
				'--persist-to',
				NodePath.join(input.paths.data, 'backend'),
				'--env-file',
				NodePath.join(input.paths.data, 'backend', '.dev.vars'),
			],
			cwd: NodePath.join(input.paths.worktree, 'apps', 'backend'),
			env: input.env,
			label: 'backend',
		},
		'account-web': {
			args: [
				'exec',
				'nuxi',
				'dev',
				'--host',
				'127.0.0.1',
				'--port',
				String(input.ports.accountWeb),
			],
			cwd: NodePath.join(input.paths.worktree, 'apps', 'frontend'),
			// The website is upstream Modrinth; these are its own settings for which API to call.
			env: {
				...input.env,
				BASE_URL: `${backendUrl}/v2/`,
				BROWSER_BASE_URL: `${backendUrl}/v2/`,
				PORT: String(input.ports.accountWeb),
			},
			label: 'account-web',
		},
		'app-frontend': {
			args: ['dev', '--port', String(input.ports.app), '--strictPort'],
			cwd: NodePath.join(input.paths.worktree, 'apps', 'app-frontend'),
			env: input.env,
			label: 'app-frontend',
		},
		core: {
			args: ['run', '--filter', '@amberite/core', 'dev'],
			cwd: input.paths.worktree,
			env: coreEnv,
			label: 'core',
		},
	}
	const shared = labels.map((label) => specs[label]!)
	if (input.mode !== 'dev' && input.mode !== 'dev:app') return shared
	return [
		...shared,
		...input.scenarios.map((scenario) =>
			createAppProcessSpec({
				branch: input.branch,
				backendUrl,
				env: input.env,
				paths: input.paths,
				scenario,
				tauriOverride,
			}),
		),
	]
}

function createAppProcessSpec(input: {
	readonly backendUrl: string
	readonly branch: string
	readonly env: NodeJS.ProcessEnv
	readonly paths: WorktreePaths
	readonly scenario: number
	readonly tauriOverride: unknown
}): ProcessSpec {
	const dataDir = scenarioDataPath(input.paths, input.scenario)
	const namespace = `${input.branch}:scenario:${input.scenario}`
	const appDevConfig = {
		backendUrl: input.backendUrl,
		authMode: 'dev',
		branch: input.branch,
		credentialNamespace: namespace,
		dataDir,
		title: `Modrinth ${input.scenario} - ${input.branch}`,
		username: scenarioUsername(input.scenario),
	}
	return {
		args: [
			'exec',
			'tauri',
			'dev',
			'--features',
			'browser-bridge',
			'--config',
			JSON.stringify(input.tauriOverride),
			'--',
			'--',
			'--amberite-dev-config',
			JSON.stringify(appDevConfig),
		],
		cwd: NodePath.join(input.paths.worktree, 'apps', 'app'),
		env: {
			...input.env,
			RUST_LOG: 'theseus=warn,theseus_gui=warn,amberite_browser_bridge=info,webview=off',
			THESEUS_CONFIG_DIR: dataDir,
			WEBVIEW2_USER_DATA_FOLDER: NodePath.join(dataDir, 'webview2'),
		},
		label: `app:${input.scenario}`,
	}
}

function scenarioDataPath(paths: WorktreePaths, scenario: number): string {
	return NodePath.join(paths.scenariosData, String(scenario))
}

function spawnProcess(worktree: string, spec: ProcessSpec): RunningProcess {
	const vpPath = resolveVpPath(worktree)
	const child = NodeChildProcess.spawn(
		spec.executable ?? process.execPath,
		spec.executable ? [...spec.args] : [vpPath, ...spec.args],
		{
			cwd: spec.cwd,
			detached: process.platform !== 'win32',
			env: spec.env,
			stdio: ['pipe', 'pipe', 'pipe'],
			windowsHide: true,
		},
	)
	runnerLog('info', `${spec.label} PID ${child.pid ?? 'unavailable'}`)
	const recentOutput: string[] = []
	const rememberOutput = (line: string) => {
		recentOutput.push(line)
		if (recentOutput.length > 20) recentOutput.shift()
	}
	pipeOutput(child.stdout, spec.label, process.stdout, rememberOutput)
	pipeOutput(child.stderr, spec.label, process.stderr, rememberOutput)

	const done = new Promise<number>((resolve, reject) => {
		child.once('error', (error) =>
			reject(new DevRunnerError(`Could not start ${spec.label}: ${error.message}`)),
		)
		child.once('close', (code, signal) => {
			if (code !== null) resolve(code)
			else resolve(signal ? 1 : 0)
		})
	})

	return { child, done, label: spec.label, recentOutput }
}

async function stopProcess(running: RunningProcess): Promise<void> {
	const pid = running.child.pid
	if (!pid || running.child.exitCode !== null) return

	if (process.platform === 'win32') {
		NodeChildProcess.spawnSync('taskkill', ['/PID', String(pid), '/T', '/F'], {
			stdio: 'ignore',
			windowsHide: true,
		})
		return
	}

	try {
		process.kill(-pid, 'SIGTERM')
	} catch {
		running.child.kill('SIGTERM')
	}
}

function pipeOutput(
	stream: NodeJS.ReadableStream | null,
	label: string,
	destination: NodeJS.WritableStream,
	onLine?: (line: string) => void,
): void {
	if (!stream) return
	const lines = NodeReadline.createInterface({ input: stream })
	let previousLine = ''
	const seenRepeatedNoise = new Set<string>()
	lines.on('line', (line) => {
		const plainLine = stripAnsi(line).trimEnd()
		onLine?.(plainLine)
		if (
			!plainLine.trim() ||
			isNoisyProgressLine(plainLine) ||
			plainLine === previousLine ||
			(isRepeatedNoiseLine(plainLine) && seenRepeatedNoise.has(plainLine))
		)
			return
		if (isRepeatedNoiseLine(plainLine)) seenRepeatedNoise.add(plainLine)
		previousLine = plainLine
		const displayedLine =
			plainLine.length > 4_000 ? `${plainLine.slice(0, 4_000)} … truncated` : line.trimEnd()
		destination.write(`[${label}] ${displayedLine}\n`)
	})
}

function appExecutableIsLocked(running: RunningProcess): boolean {
	return (
		running.recentOutput.some(
			(line) => line.includes('failed to remove file') && line.includes('theseus_gui.exe'),
		) && running.recentOutput.some((line) => line.includes('Access is denied. (os error 5)'))
	)
}

function isWindowsProcessRunning(imageName: string): boolean {
	if (process.platform !== 'win32') return false
	const result = NodeChildProcess.spawnSync(
		'tasklist',
		['/FI', `IMAGENAME eq ${imageName}`, '/FO', 'CSV', '/NH'],
		{ encoding: 'utf8', windowsHide: true },
	)
	return result.status === 0 && result.stdout.toLowerCase().includes(`"${imageName.toLowerCase()}"`)
}

function createCommandInput(input: {
	readonly getCore: () => RunningProcess | undefined
	readonly restart: (target: string) => Promise<void>
	readonly stop: () => Promise<void>
}): NodeReadline.Interface {
	const commands = NodeReadline.createInterface({ input: process.stdin })
	commands.on('line', (line) => {
		const command = line.trim()
		if (!command) return
		if (command === 'quit') {
			void input.stop()
			return
		}
		const restart = command.match(/^rs\s+(core|\d+)$/i)
		if (restart) {
			void input.restart(restart[1].toLowerCase())
			return
		}
		const coreCommand = command.match(/^core\s+(.+)$/i)
		if (coreCommand) {
			const core = input.getCore()
			if (!core?.child.stdin?.writable) {
				runnerLog('warning', 'Core is not running.')
				return
			}
			core.child.stdin.write(`${coreCommand[1]}\n`)
			return
		}
		if (command === 'help') {
			runnerLog('info', 'Commands: rs <scenario>, rs core, core <command>, quit')
			return
		}
		runnerLog('warning', `Unknown command ${command}. Type help for available commands.`)
	})
	return commands
}

function isNoisyProgressLine(line: string): boolean {
	return (
		/^\s*Info Watching .+ for changes\.\.\.$/.test(line) ||
		/^\s*Info `(?:tauri|tauri-build)` dependency has workspace inheritance enabled\./.test(line) ||
		/^\s*Running DevCommand \(`/.test(line) ||
		/^\s*(?:Finished|Running)\s+/.test(line) ||
		/^\s*(?:Building|Checking|Fetch)\s+\[[= >-]+\]/.test(line) ||
		/^Browserslist: browsers data \(caniuse-lite\) is .+ old\./.test(line) ||
		/^\s*npx update-browserslist-db@latest$/.test(line) ||
		/^\s*Why you should do it regularly: https:\/\/github\.com\/browserslist\/update-db#readme$/.test(
			line,
		) ||
		/^\s*➜\s+press h \+ enter to show help$/.test(line)
	)
}

function isRepeatedNoiseLine(line: string): boolean {
	return (
		line.includes('Skipping origin header as it is a forbidden header') ||
		line.includes('if keeping the header is a desired behavior')
	)
}

function stripAnsi(value: string): string {
	return NodeUtil.stripVTControlCharacters(value)
}

function runnerLog(level: LogLevel, message: string): void {
	const destination = level === 'error' ? process.stderr : process.stdout
	destination.write(`[dev-runner] ${message}\n`)
}

async function portIsAvailable(port: number, hosts: ReadonlyArray<string>): Promise<boolean> {
	for (const host of hosts) {
		if (!(await canListen(port, host))) return false
	}
	return true
}

function canListen(port: number, host: string): Promise<boolean> {
	return new Promise((resolve) => {
		const server = NodeNet.createServer()
		server.unref()
		server.once('error', () => resolve(false))
		server.listen({ host, port }, () => server.close(() => resolve(true)))
	})
}

function requiredPortNames(mode: DevMode): ReadonlyArray<PortName> {
	switch (mode) {
		case 'dev':
			return ['app', 'backend', 'accountWeb', 'core']
		case 'dev:backend':
			return ['backend', 'accountWeb']
		case 'dev:app':
			return ['app']
		case 'dev:core':
			return ['core']
	}
}

function stableHash(value: string): number {
	return NodeCrypto.createHash('sha256').update(value).digest().readUInt32BE(0)
}

function samePath(left: string, right: string): boolean {
	const normalize = (value: string) => {
		const resolved = NodePath.resolve(value)
		return process.platform === 'win32' ? resolved.toLowerCase() : resolved
	}
	return normalize(left) === normalize(right)
}

function currentBranch(worktree: string): string {
	return git(['branch', '--show-current'], worktree) || NodePath.basename(worktree)
}

function git(args: ReadonlyArray<string>, cwd: string): string {
	const result = NodeChildProcess.spawnSync('git', args, { cwd, encoding: 'utf8' })
	if (result.status !== 0) {
		throw new DevRunnerError(result.stderr.trim() || `git ${args.join(' ')} failed.`)
	}
	return result.stdout.trim()
}

function readEnv(path: string): NodeJS.ProcessEnv {
	if (!NodeFS.existsSync(path)) return {}
	return NodeUtil.parseEnv(NodeFS.readFileSync(path, 'utf8'))
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function prepareLocalBackend(paths: WorktreePaths, env: NodeJS.ProcessEnv): void {
	const variables = `LOCAL_DEV=true\nLOCAL_DEV_SECRET=${env.AMBERITE_LOCAL_DEV_SECRET}\n`
	NodeFS.writeFileSync(NodePath.join(paths.data, 'backend', '.dev.vars'), variables, {
		mode: 0o600,
	})
	const result = NodeChildProcess.spawnSync(
		process.execPath,
		[
			resolveVpPath(paths.worktree),
			'exec',
			'wrangler',
			'd1',
			'migrations',
			'apply',
			'accounts',
			'--local',
			'--persist-to',
			NodePath.join(paths.data, 'backend'),
		],
		{
			cwd: NodePath.join(paths.worktree, 'apps', 'backend'),
			env,
			stdio: 'inherit',
			windowsHide: true,
		},
	)
	if (result.status !== 0) throw new DevRunnerError('Local account database migration failed.')
}

function resolveVpPath(worktree: string): string {
	const path = NodePath.join(worktree, 'node_modules', 'vite-plus', 'bin', 'vp')
	if (!NodeFS.existsSync(path)) {
		throw new DevRunnerError('Vite+ is not installed. Run vp install first.')
	}
	return path
}

function writeRuntimeFile(input: {
	readonly branch: string
	readonly mode: DevMode
	readonly paths: WorktreePaths
	readonly ports: DevPorts
	readonly scenarios: ReadonlyArray<number>
	readonly source: string
}): void {
	const content = {
		branch: input.branch,
		dataDir: input.paths.data,
		mode: input.mode,
		ports: input.ports,
		scenarios: input.scenarios.map((scenario) => ({
			dataDir: scenarioDataPath(input.paths, scenario),
			number: scenario,
			username: scenarioUsername(scenario),
		})),
		source: input.source,
		urls: {
			backend: `http://127.0.0.1:${input.ports.backend}`,
			accountWeb: `http://127.0.0.1:${input.ports.accountWeb}`,
			app: `http://localhost:${input.ports.app}`,
			core: `http://127.0.0.1:${input.ports.core}`,
		},
	}
	NodeFS.writeFileSync(input.paths.runtime, `${JSON.stringify(content, null, '\t')}\n`)
}

function printPlan(input: {
	readonly branch: string
	readonly mode: DevMode
	readonly paths: WorktreePaths
	readonly ports: DevPorts
	readonly scenarios: ReadonlyArray<number>
	readonly source: string
	readonly specs: ReadonlyArray<ProcessSpec>
}): void {
	runnerLog('info', `${input.branch} · ${input.mode}`)
	runnerLog('info', `data ${input.paths.data}`)
	runnerLog('info', `ports from ${input.source}`)
	const labels = input.specs.map((spec) => spec.label)
	if (labels.includes('app-frontend')) runnerLog('info', `App http://localhost:${input.ports.app}`)
	if (labels.includes('backend')) {
		runnerLog('info', `Accounts and sharing http://127.0.0.1:${input.ports.backend} (local)`)
		runnerLog('info', `Account sign-in http://127.0.0.1:${input.ports.accountWeb}`)
	}
	if (labels.includes('core')) runnerLog('info', `Core http://127.0.0.1:${input.ports.core}`)
	if (input.scenarios.length > 0) {
		runnerLog('info', `scenarios ${input.scenarios.join(', ')}`)
	}
	runnerLog('info', `processes ${input.specs.map((spec) => spec.label).join(', ')}`)
}

const isMain = process.argv[1] && samePath(process.argv[1], NodeURL.fileURLToPath(import.meta.url))
if (isMain) {
	main().catch((error: unknown) => {
		runnerLog('error', error instanceof Error ? error.message : String(error))
		process.exitCode = 1
	})
}
