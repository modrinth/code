import { execFileSync, spawnSync } from 'child_process'
import * as fs from 'fs'
import { createInterface } from 'readline/promises'
import { stdin as input, stdout as output } from 'process'
import chalk from 'chalk'

const CHANGELOG_PATH = 'packages/blog/changelog.ts'
const BUILD_WORKFLOW = 'theseus-build.yml'
const RELEASE_WORKFLOW = 'theseus-release.yml'
const WORKFLOW_POLL_MS = 30_000
const WORKFLOW_TIMEOUT_MS = 1000 * 60 * 60 * 3

type Product = 'web' | 'app' | 'hosting'

interface Args {
	dryRun: boolean
}

interface ParsedChangelogEntry {
	date?: string
	product: Product
	version?: string
}

interface GhRunListItem {
	databaseId: number
	status: string
	conclusion: string | null
	url: string
	headBranch: string
	event: string
	createdAt: string
}

interface GhRunView {
	status: string
	conclusion: string | null
	url: string
	jobs: Array<{
		name: string
		status: string
		conclusion: string | null
		steps?: Array<{
			name: string
			status: string
			conclusion: string | null
		}>
	}>
}

function parseArgs(argv: string[]): Args {
	let dryRun = false

	for (const arg of argv) {
		if (arg === '--') {
			continue
		}
		if (arg === '--dry-run') {
			dryRun = true
			continue
		}

		console.error(chalk.red(`Unknown argument: ${arg}`))
		process.exit(1)
	}

	return { dryRun }
}

function git(args: string[]): string {
	return execFileSync('git', args, { encoding: 'utf-8' }).trim()
}

function gitInherit(args: string[]): void {
	execFileSync('git', args, { stdio: 'inherit' })
}

function ensureMainBranch(dryRun: boolean): void {
	const branch = git(['branch', '--show-current'])
	if (branch !== 'main') {
		if (dryRun) {
			console.warn(
				chalk.yellow(
					`Dry run: release:push normally runs from main, currently on ${branch || 'detached HEAD'}`,
				),
			)
			return
		}

		console.error(chalk.red(`release:push must be run from main, currently on ${branch || 'detached HEAD'}`))
		process.exit(1)
	}
}

function ensureMainMatchesOrigin(): void {
	gitInherit(['fetch', 'origin', '--tags'])

	const head = git(['rev-parse', 'HEAD'])
	const originMain = git(['rev-parse', 'origin/main'])
	if (head !== originMain) {
		console.error(chalk.red('main must match origin/main before pushing a release changelog'))
		console.error(chalk.gray(`HEAD:        ${head}`))
		console.error(chalk.gray(`origin/main: ${originMain}`))
		process.exit(1)
	}
}

function statusPaths(): string[] {
	const status = git(['status', '--porcelain=v1'])
	if (!status) return []

	return status.split('\n').map((line) => {
		const rawPath = line.slice(3).trim()
		const renamedPath = rawPath.includes(' -> ') ? rawPath.split(' -> ').at(-1)! : rawPath
		return renamedPath.replace(/^"|"$/g, '')
	})
}

function ensureOnlyChangelogChanged(dryRun: boolean): void {
	const paths = statusPaths()
	if (paths.length === 0) {
		console.error(chalk.red('No changelog changes found to push'))
		process.exit(1)
	}

	const invalidPaths = paths.filter((changedPath) => changedPath !== CHANGELOG_PATH)
	if (invalidPaths.length > 0) {
		if (dryRun) {
			console.warn(chalk.yellow(`Dry run: release:push normally only allows ${CHANGELOG_PATH} changes`))
			for (const invalidPath of invalidPaths) {
				console.warn(chalk.gray(`- ${invalidPath}`))
			}
			return
		}

		console.error(chalk.red(`release:push only commits ${CHANGELOG_PATH}`))
		for (const invalidPath of invalidPaths) {
			console.error(chalk.gray(`- ${invalidPath}`))
		}
		process.exit(1)
	}
}

function parseChangelogEntries(source: string): ParsedChangelogEntry[] {
	const stringPattern = String.raw`(?:\\` + '`' + String.raw`|[^` + '`' + String.raw`])*`
	const entryRegex = new RegExp(
		String.raw`\{\s*(?:date:\s*` +
			'`' +
			`(${stringPattern})` +
			'`' +
			String.raw`,\s*)?product:\s*'(\w+)',(?:\s*version:\s*['` +
			'`' +
			String.raw`]([^'` +
			'`' +
			String.raw`]+)['` +
			'`' +
			String.raw`],)?\s*body:\s*` +
			'`' +
			`(${stringPattern})` +
			'`' +
			String.raw`,\s*\}`,
		'g',
	)

	const entries: ParsedChangelogEntry[] = []
	let match: RegExpExecArray | null
	while ((match = entryRegex.exec(source)) !== null) {
		entries.push({
			date: match[1],
			product: match[2] as Product,
			version: match[3],
		})
	}

	return entries
}

function detectNewAppVersion(baseRef: string): string | null {
	const base = git(['show', `${baseRef}:${CHANGELOG_PATH}`])
	const current = fs.readFileSync(CHANGELOG_PATH, 'utf-8')

	const baseAppVersions = new Set(
		parseChangelogEntries(base)
			.filter((entry) => entry.product === 'app' && entry.version)
			.map((entry) => entry.version!),
	)
	const newAppEntries = parseChangelogEntries(current).filter(
		(entry) => entry.product === 'app' && entry.version && !baseAppVersions.has(entry.version),
	)

	if (newAppEntries.length === 0) {
		return null
	}

	if (newAppEntries.length > 1) {
		console.error(chalk.red('release:push found multiple new App changelog entries'))
		for (const entry of newAppEntries) {
			console.error(chalk.gray(`- ${entry.version}`))
		}
		process.exit(1)
	}

	return newAppEntries[0].version!
}

function ensureTagDoesNotExist(version: string): void {
	const tag = `v${version}`

	try {
		git(['rev-parse', '--verify', `refs/tags/${tag}`])
		console.error(chalk.red(`Tag ${tag} already exists locally`))
		process.exit(1)
	} catch {
		// Missing local tag is expected.
	}

	const remote = spawnSync('git', ['ls-remote', '--exit-code', '--tags', 'origin', `refs/tags/${tag}`], {
		stdio: 'ignore',
	})
	if (remote.status === 0) {
		console.error(chalk.red(`Tag ${tag} already exists on origin`))
		process.exit(1)
	}
	if (remote.status !== 2) {
		console.error(chalk.red(`Could not check remote tag ${tag}`))
		process.exit(remote.status ?? 1)
	}
}

async function confirmTagPush(version: string): Promise<boolean> {
	const tag = `v${version}`
	const rl = createInterface({ input, output })
	try {
		const answer = await rl.question(`Create and push app release tag ${tag}? [y/N] `)
		return answer.trim().toLowerCase() === 'y' || answer.trim().toLowerCase() === 'yes'
	} finally {
		rl.close()
	}
}

function runGhJson<T>(args: string[]): T {
	const output = execFileSync('gh', args, { encoding: 'utf-8' }).trim()
	return JSON.parse(output || 'null') as T
}

function ensureGhAvailable(): void {
	try {
		execFileSync('gh', ['--version'], { stdio: 'ignore' })
	} catch {
		console.error(chalk.red('GitHub CLI is required to wait for the app release workflows'))
		console.error(chalk.gray('Install gh and run `gh auth login`, then re-run release:push if needed.'))
		process.exit(1)
	}
}

function findWorkflowRun(workflow: string, tag: string, event: string): GhRunListItem | null {
	const runs = runGhJson<GhRunListItem[]>([
		'run',
		'list',
		'--workflow',
		workflow,
		'--json',
		'databaseId,status,conclusion,url,headBranch,event,createdAt',
		'--limit',
		'20',
	])

	const matches = runs
		.filter((run) => run.headBranch === tag && run.event === event)
		.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())

	return matches[0] ?? null
}

function viewWorkflowRun(runId: number): GhRunView {
	return runGhJson<GhRunView>([
		'run',
		'view',
		String(runId),
		'--json',
		'status,conclusion,url,jobs',
	])
}

function formatDuration(ms: number): string {
	const totalSeconds = Math.floor(ms / 1000)
	const minutes = Math.floor(totalSeconds / 60)
	const seconds = totalSeconds % 60
	return `${minutes}m ${seconds.toString().padStart(2, '0')}s`
}

function describeRun(run: GhRunView): string {
	const activeJob =
		run.jobs.find((job) => job.status === 'in_progress') ??
		run.jobs.find((job) => job.status === 'queued') ??
		run.jobs.find((job) => job.status !== 'completed')

	if (!activeJob) {
		return 'completed'
	}

	const activeStep =
		activeJob.steps?.find((step) => step.status === 'in_progress') ??
		activeJob.steps?.find((step) => step.status === 'queued') ??
		activeJob.steps?.find((step) => step.status !== 'completed')

	return activeStep ? `${activeJob.name} / ${activeStep.name}` : activeJob.name
}

async function sleep(ms: number): Promise<void> {
	await new Promise((resolve) => setTimeout(resolve, ms))
}

async function waitForWorkflow(workflow: string, tag: string, event: string, label: string): Promise<void> {
	const startedAt = Date.now()
	let runId: number | null = null
	let runUrl: string | null = null

	while (Date.now() - startedAt < WORKFLOW_TIMEOUT_MS) {
		const elapsed = formatDuration(Date.now() - startedAt)

		if (!runId) {
			const run = findWorkflowRun(workflow, tag, event)
			if (run) {
				runId = run.databaseId
				runUrl = run.url
				console.log(chalk.cyan(`[${elapsed}] ${label}: found run ${runUrl}`))
			} else {
				console.log(chalk.gray(`[${elapsed}] ${label}: waiting for workflow run`))
				await sleep(WORKFLOW_POLL_MS)
				continue
			}
		}

		const run = viewWorkflowRun(runId)
		runUrl = run.url || runUrl

		if (run.status === 'completed') {
			if (run.conclusion === 'success') {
				console.log(chalk.green(`[${elapsed}] ${label}: completed successfully`))
				return
			}

			console.error(chalk.red(`[${elapsed}] ${label}: completed with ${run.conclusion ?? 'unknown result'}`))
			if (runUrl) {
				console.error(chalk.gray(runUrl))
			}
			process.exit(1)
		}

		console.log(chalk.cyan(`[${elapsed}] ${label}: ${run.status} - ${describeRun(run)}`))
		await sleep(WORKFLOW_POLL_MS)
	}

	console.error(chalk.red(`${label} did not finish within ${formatDuration(WORKFLOW_TIMEOUT_MS)}`))
	if (runUrl) {
		console.error(chalk.gray(runUrl))
	}
	process.exit(1)
}

function playChime(): void {
	process.stdout.write('\u0007')

	if (process.platform === 'darwin') {
		spawnSync('afplay', ['/System/Library/Sounds/Glass.aiff'], { stdio: 'ignore' })
	}
}

function printDryRunPlan(appVersion: string | null): void {
	const commitMessage = appVersion ? `Add changelog for ${appVersion}` : 'Add changelog entries'

	console.log(chalk.yellow('Dry run: no git add, commit, tag, push, or workflow polling will run'))
	console.log(chalk.gray(`Would stage ${CHANGELOG_PATH}`))
	console.log(chalk.gray(`Would commit with message: ${commitMessage}`))
	console.log(chalk.gray('Would push main to origin/main'))

	if (appVersion) {
		const tag = `v${appVersion}`
		console.log(chalk.gray(`Would ask to create and push annotated tag ${tag}`))
		console.log(chalk.gray(`Would wait for ${BUILD_WORKFLOW} and ${RELEASE_WORKFLOW}`))
	}

	console.log()
	console.log(chalk.green('When ready:'))
	console.log('git push origin main:prod')
}

async function main(): Promise<void> {
	const args = parseArgs(process.argv.slice(2))

	ensureMainBranch(args.dryRun)
	ensureOnlyChangelogChanged(args.dryRun)
	if (!args.dryRun) {
		ensureMainMatchesOrigin()
	}

	const baseRef = args.dryRun ? 'HEAD' : 'origin/main'
	const appVersion = detectNewAppVersion(baseRef)
	if (appVersion && !args.dryRun) {
		ensureTagDoesNotExist(appVersion)
	}

	if (args.dryRun) {
		printDryRunPlan(appVersion)
		return
	}

	const commitMessage = appVersion ? `Add changelog for ${appVersion}` : 'Add changelog entries'
	gitInherit(['add', CHANGELOG_PATH])
	gitInherit(['commit', '-m', commitMessage])
	gitInherit(['push', 'origin', 'main'])

	if (appVersion) {
		const shouldPushTag = await confirmTagPush(appVersion)
		if (!shouldPushTag) {
			console.log(chalk.yellow('Skipped app release tag creation'))
			console.log()
			console.log(chalk.green('When ready:'))
			console.log('git push origin main:prod')
			return
		}

		ensureGhAvailable()

		const tag = `v${appVersion}`
		gitInherit(['tag', '-a', tag, '-m', tag])
		gitInherit(['push', 'origin', 'tag', tag])

		await waitForWorkflow(BUILD_WORKFLOW, tag, 'push', 'App build')
		await waitForWorkflow(RELEASE_WORKFLOW, tag, 'workflow_run', 'App release')
		playChime()
	}

	console.log()
	console.log(chalk.green('When ready:'))
	console.log('git push origin main:prod')
}

main().catch((error) => {
	console.error(error)
	process.exit(1)
})
