import { describe, expect, it } from 'vite-plus/test'

import {
	createProcessSpecs,
	createRuntimeEnvironment,
	findFirstAvailableOffset,
	isBrowserAllowedPort,
	parseInput,
	portsForOffset,
	processLabelsForMode,
	resolveStartOffset,
	scenarioUsername,
} from './dev-runner.ts'

describe('dev runner ports', () => {
	it('keeps the documented ports in the primary checkout', () => {
		expect(
			resolveStartOffset({
				primaryPath: '/repo',
				worktreePath: '/repo',
			}),
		).toEqual({ offset: 0, source: 'primary checkout' })
		expect(portsForOffset(0)).toEqual({
			app: 1420,
			backend: 8787,
			accountWeb: 3100,
			core: 16662,
		})
	})

	it('gives a worktree a stable non-zero offset', () => {
		const first = resolveStartOffset({
			primaryPath: '/repo',
			worktreePath: '/repo-worktrees/feature',
		})
		const second = resolveStartOffset({
			primaryPath: '/repo',
			worktreePath: '/repo-worktrees/feature',
		})

		expect(first).toEqual(second)
		expect(first.offset).toBeGreaterThan(0)
	})

	it('honours explicit offsets and named development instances', () => {
		expect(
			resolveStartOffset({
				explicitOffset: '42',
				primaryPath: '/repo',
				worktreePath: '/repo',
			}),
		).toEqual({ offset: 42, source: 'AMBERITE_PORT_OFFSET=42' })

		const named = resolveStartOffset({
			devInstance: 'design-review',
			primaryPath: '/repo',
			worktreePath: '/repo',
		})
		expect(named.offset).toBeGreaterThan(0)
		expect(named.source).toBe('hashed AMBERITE_DEV_INSTANCE=design-review')
	})

	it('rejects invalid explicit offsets', () => {
		expect(() =>
			resolveStartOffset({
				explicitOffset: '-1',
				primaryPath: '/repo',
				worktreePath: '/repo',
			}),
		).toThrow('AMBERITE_PORT_OFFSET must be a non-negative integer')
	})

	it('moves the complete stack together when a port is occupied', async () => {
		const checked: number[] = []
		const offset = await findFirstAvailableOffset({
			checkPort: async (port) => {
				checked.push(port)
				return port !== 1420
			},
			mode: 'dev',
			startOffset: 0,
		})

		expect(offset).toBe(1)
		expect(checked).toContain(1420)
		expect(portsForOffset(offset)).toEqual({
			app: 1421,
			backend: 8788,
			accountWeb: 3101,
			core: 16663,
		})
	})

	it('checks only the ports started by a partial mode', async () => {
		const checked: number[] = []
		await findFirstAvailableOffset({
			checkPort: async (port) => {
				checked.push(port)
				return true
			},
			mode: 'dev:core',
			startOffset: 0,
		})

		expect(checked).toEqual([16662])
	})

	it('skips ports blocked by browsers', () => {
		expect(isBrowserAllowedPort(6000)).toBe(false)
		expect(isBrowserAllowedPort(16662)).toBe(true)
	})
})

describe('dev runner environment', () => {
	it('owns runtime URLs and removes ambient host and port overrides', () => {
		const env = createRuntimeEnvironment({
			baseEnv: {
				HOST: 'example.com',
				PORT: '9999',
				SHARED: 'kept',
			},
			paths: {
				coreData: '/repo/.data/core',
				data: '/repo/.data',
				primary: '/repo',
				runtime: '/repo/.data/runtime.json',
				scenariosData: '/repo/.data/scenarios',
				worktree: '/repo',
			},
			ports: portsForOffset(3),
		})

		expect(env.HOST).toBeUndefined()
		expect(env.PORT).toBeUndefined()
		expect(env.SHARED).toBe('kept')
		expect(env.AMBERITE_DEV_MODE).toBe('true')
		expect(env.ACCOUNT_API_URL).toBe('http://127.0.0.1:8790')
	})
})

describe('dev runner modes', () => {
	it('runs the complete product in dev mode', () => {
		expect(processLabelsForMode('dev')).toEqual(['backend', 'account-web', 'core', 'app-frontend'])
	})

	it('keeps partial modes focused', () => {
		expect(processLabelsForMode('dev:app')).toEqual(['app-frontend'])
		expect(processLabelsForMode('dev:backend')).toEqual(['backend', 'account-web'])
		expect(processLabelsForMode('dev:core')).toEqual(['core'])
	})
})

describe('dev runner scenarios', () => {
	it('uses dev.json defaults when no scenario is passed', () => {
		expect(parseInput(['dev'], [2, 4])).toEqual({
			dryRun: false,
			mode: 'dev',
			scenarios: [2, 4],
		})
	})

	it('accepts several scenarios and removes duplicates', () => {
		expect(parseInput(['dev', '1', '2', '1'], [4]).scenarios).toEqual([1, 2])
		expect(parseInput(['dev:app', '3'], [1]).scenarios).toEqual([3])
	})

	it('does not attach scenarios to backend-only modes', () => {
		expect(parseInput(['dev:core'], [1])).toEqual({
			dryRun: false,
			mode: 'dev:core',
			scenarios: [],
		})
		expect(() => parseInput(['dev:core', '2'], [1])).toThrow(
			'dev:core does not start App scenarios',
		)
	})

	it('derives the fake account from the scenario number', () => {
		expect(scenarioUsername(12)).toBe('scenario_12')
	})

	it('gives each App isolated persistent state', () => {
		const paths = {
			coreData: `${process.cwd()}/.data/core`,
			data: `${process.cwd()}/.data`,
			primary: process.cwd(),
			runtime: `${process.cwd()}/.data/runtime.json`,
			scenariosData: `${process.cwd()}/.data/scenarios`,
			worktree: process.cwd(),
		}
		const specs = createProcessSpecs({
			branch: 'feature',
			env: {},
			mode: 'dev',
			paths,
			ports: portsForOffset(0),
			scenarios: [1, 3],
		})

		expect(specs.map((spec) => spec.label)).toEqual([
			'backend',
			'account-web',
			'core',
			'app-frontend',
			'app:1',
			'app:3',
		])
		const apps = specs.filter((spec) => spec.label.startsWith('app:'))
		for (const app of apps) {
			const config = JSON.parse(app.args[app.args.indexOf('--config') + 1]!)
			expect(config.app.security.csp['img-src']).toContain('http://127.0.0.1:8787')
			expect(config.app.security.capabilities).toEqual([
				'ads',
				'core',
				'plugins',
				{
					identifier: 'local-account-backend',
					windows: ['main'],
					permissions: [
						{
							identifier: 'http:default',
							allow: [{ url: 'http://127.0.0.1:8787/*' }, { url: 'http://127.0.0.1:16662/*' }],
						},
					],
				},
			])
		}
		const cwd = process.cwd().replaceAll('\\', '/')
		expect(apps.map((spec) => spec.env.THESEUS_CONFIG_DIR?.replaceAll('\\', '/'))).toEqual([
			`${cwd}/.data/scenarios/1`,
			`${cwd}/.data/scenarios/3`,
		])
		expect(apps.map((spec) => JSON.parse(spec.args.at(-1)!).username)).toEqual([
			'scenario_1',
			'scenario_3',
		])
	})
})
