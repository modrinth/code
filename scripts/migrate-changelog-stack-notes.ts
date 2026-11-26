import { execSync } from 'child_process'
import chalk from 'chalk'

const GITHUB_API = 'https://api.github.com'
const REPO = 'modrinth/code'
const CHANGELOG_MARKER = '<!-- changelog -->'
const STACK_NOTE_MARKER = '<!-- changelog-merged -->'
const STACK_NOTE = /<!-- changelog-merged -->[ \t]*\r?\n(?:>.*(?:\r?\n|$))*(?:[ \t]*\r?\n)*/g
const CHANGELOG_SYSTEM_INTRODUCED = '2026-03-23T00:00:00Z'

interface IssueComment {
	id: number
	body: string
	issue_url: string
}

function parseArgs(argv: string[]): { dryRun: boolean } {
	let dryRun = false

	for (const arg of argv) {
		if (arg === '--') continue
		if (arg === '--dry-run') {
			dryRun = true
		} else {
			console.error(chalk.red(`Unknown argument: ${arg}`))
			process.exit(1)
		}
	}

	return { dryRun }
}

function getToken(): string {
	if (process.env.GITHUB_TOKEN) return process.env.GITHUB_TOKEN

	try {
		return execSync('gh auth token', { encoding: 'utf-8' }).trim()
	} catch {
		console.error(
			chalk.red(
				'GITHUB_TOKEN not set and `gh auth token` failed. Run `gh auth login` or set GITHUB_TOKEN.',
			),
		)
		process.exit(1)
	}
}

async function githubFetch(
	endpoint: string,
	token: string,
	options?: RequestInit,
): Promise<Response> {
	return fetch(`${GITHUB_API}${endpoint}`, {
		...options,
		headers: {
			Authorization: `token ${token}`,
			Accept: 'application/vnd.github.v3+json',
			...options?.headers,
		},
	})
}

async function fetchStackNoteComments(token: string): Promise<IssueComment[]> {
	const matches: IssueComment[] = []
	let page = 1

	while (true) {
		const res = await githubFetch(
			`/repos/${REPO}/issues/comments?since=${CHANGELOG_SYSTEM_INTRODUCED}&sort=created&direction=asc&per_page=100&page=${page}`,
			token,
		)

		if (!res.ok) {
			console.error(chalk.red(`GitHub API error: ${res.status} ${res.statusText}`))
			process.exit(1)
		}

		const data = (await res.json()) as IssueComment[]
		matches.push(
			...data.filter(
				(c) => c.body?.includes(CHANGELOG_MARKER) && c.body.includes(STACK_NOTE_MARKER),
			),
		)

		if (data.length < 100) break
		page++
	}

	return matches
}

async function main() {
	const args = parseArgs(process.argv.slice(2))
	const token = getToken()

	const comments = await fetchStackNoteComments(token)
	console.log(chalk.gray(`Found ${comments.length} changelog comment(s) with a stack note`))

	for (const comment of comments) {
		const prNumber = comment.issue_url.split('/').pop()
		const body = comment.body.replace(STACK_NOTE, '')

		if (args.dryRun) {
			console.log(chalk.cyan(`[dry-run] PR #${prNumber}: would remove stack note`))
			continue
		}

		const res = await githubFetch(`/repos/${REPO}/issues/comments/${comment.id}`, token, {
			method: 'PATCH',
			body: JSON.stringify({ body }),
		})

		if (res.ok) {
			console.log(chalk.green(`PR #${prNumber}: removed stack note`))
		} else {
			console.error(
				chalk.red(`PR #${prNumber}: failed to update (${res.status} ${res.statusText})`),
			)
		}
	}
}

main()
