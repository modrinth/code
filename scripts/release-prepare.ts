import { execFileSync, spawnSync } from 'child_process'
import chalk from 'chalk'

const CHANGELOG_PATH = 'packages/blog/changelog.ts'

interface Args {
	version?: string
	dryRun: boolean
}

function parseArgs(argv: string[]): Args {
	let version: string | undefined
	let dryRun = false

	for (let i = 0; i < argv.length; i++) {
		const arg = argv[i]
		if (arg === '--') {
			continue
		}
		if (arg === '--version') {
			const value = argv[++i]
			if (!value || value.startsWith('--')) {
				console.error(chalk.red('--version requires a value'))
				process.exit(1)
			}
			version = value.replace(/^v/, '')
			continue
		}
		if (arg === '--dry-run') {
			dryRun = true
			continue
		}

		console.error(chalk.red(`Unknown argument: ${arg}`))
		process.exit(1)
	}

	return { version, dryRun }
}

function git(args: string[]): string {
	return execFileSync('git', args, { encoding: 'utf-8' }).trim()
}

function gitInherit(args: string[]): void {
	execFileSync('git', args, { stdio: 'inherit' })
}

function ensureMainBranch(dryRun: boolean): boolean {
	const branch = git(['branch', '--show-current'])
	if (branch !== 'main') {
		if (dryRun) {
			console.warn(
				chalk.yellow(
					`Dry run: release:prepare normally runs from main, currently on ${branch || 'detached HEAD'}`,
				),
			)
			return false
		}

		console.error(chalk.red(`release:prepare must be run from main, currently on ${branch || 'detached HEAD'}`))
		process.exit(1)
	}

	return true
}

function ensureCleanWorktree(dryRun: boolean): void {
	const status = git(['status', '--porcelain=v1'])
	if (status) {
		if (dryRun) {
			console.warn(chalk.yellow('Dry run: release:prepare normally requires a clean worktree'))
			console.warn(chalk.gray(status))
			return
		}

		console.error(chalk.red('release:prepare requires a clean worktree'))
		console.error(status)
		process.exit(1)
	}
}

function ensureMainMatchesOrigin(): void {
	gitInherit(['fetch', 'origin', '--tags'])

	const head = git(['rev-parse', 'HEAD'])
	const originMain = git(['rev-parse', 'origin/main'])
	if (head !== originMain) {
		console.error(chalk.red('main must match origin/main before preparing a release'))
		console.error(chalk.gray(`HEAD:        ${head}`))
		console.error(chalk.gray(`origin/main: ${originMain}`))
		process.exit(1)
	}
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

function runCollector(args: Args): void {
	const collectorArgs = ['scripts/run.mjs', 'collect-changelog']
	if (args.version) {
		collectorArgs.push('--version', args.version)
	}
	if (args.dryRun) {
		collectorArgs.push('--no-bake')
	}

	const result = spawnSync(process.execPath, collectorArgs, { stdio: 'inherit' })
	if (result.status !== 0) {
		process.exit(result.status ?? 1)
	}
}

function stageChangelog(): void {
	gitInherit(['add', CHANGELOG_PATH])
}

function main(): void {
	const args = parseArgs(process.argv.slice(2))

	if (args.dryRun) {
		console.log(chalk.yellow('Dry run: changelog changes will be written and staged, but PR comments will not be baked'))
	}

	const isMainBranch = ensureMainBranch(args.dryRun)
	ensureCleanWorktree(args.dryRun)
	if (args.dryRun && !isMainBranch) {
		console.warn(chalk.yellow('Dry run: skipping origin/main freshness check on non-main branch'))
	} else {
		ensureMainMatchesOrigin()
	}
	if (args.version) {
		ensureTagDoesNotExist(args.version)
	}

	runCollector(args)
	stageChangelog()

	const staged = git(['diff', '--cached', '--name-only'])
	if (!staged.split('\n').filter(Boolean).includes(CHANGELOG_PATH)) {
		console.log(chalk.yellow('No changelog changes were staged'))
		return
	}

	console.log(chalk.green(`Staged ${CHANGELOG_PATH}`))
	if (args.dryRun) {
		console.log(chalk.yellow('Dry run complete: review or reset the staged changelog changes when done testing'))
	}
}

main()
