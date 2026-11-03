const {
	BUILD_ENVIRONMENT,
	GITHUB_API_URL,
	GITHUB_REF_NAME,
	GITHUB_REPOSITORY,
	GITHUB_RUN_ID,
	GITHUB_SERVER_URL,
	GITHUB_SHA,
	GITHUB_TOKEN,
	GITHUB_TRIGGERING_ACTOR,
	SLACK_WEBHOOK_URL,
} = process.env

/** Job and artifact names must match the matrix in `.github/workflows/theseus-build.yml`. */
const PLATFORMS = [
	{
		label: 'Windows',
		job: 'Build (namespace-profile-medium-amd64-windows)',
		artifact: 'App bundle (x86_64-pc-windows-msvc)',
	},
	{
		label: 'MacOS',
		job: 'Build (namespace-profile-medium-amd64-macos)',
		artifact: 'App bundle (universal-apple-darwin)',
	},
	{
		label: 'Linux',
		job: 'Build (namespace-profile-medium-amd64)',
		artifact: 'App bundle (x86_64-unknown-linux-gnu)',
	},
]

const runUrl = `${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}`

async function fetchRunResource(resource) {
	const response = await fetch(
		`${GITHUB_API_URL}/repos/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}/${resource}?per_page=100`,
		{
			headers: {
				Accept: 'application/vnd.github+json',
				Authorization: `Bearer ${GITHUB_TOKEN}`,
			},
		},
	)
	if (!response.ok) {
		throw new Error(`Failed to fetch ${resource}: ${response.status} ${await response.text()}`)
	}
	return response.json()
}

/** Slack treats these characters as control sequences, even inside inline code. */
function escapeMrkdwn(text) {
	return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function mrkdwnSection(text) {
	return { type: 'section', text: { type: 'mrkdwn', text } }
}

function mrkdwnContext(text) {
	return { type: 'context', elements: [{ type: 'mrkdwn', text }] }
}

/** A build without an artifact is reported as failed, as there is nothing to download. */
function summarizeBuild({ label, job: jobName, artifact: artifactName }, jobs, artifacts) {
	const job = jobs.find(({ name }) => name === jobName)
	if (!job) {
		throw new Error(`No job named "${jobName}" in this run`)
	}

	const artifact = artifacts.find(({ name }) => name === artifactName)
	if (job.conclusion !== 'success' || !artifact) {
		const failedStep = job.steps.find(({ conclusion }) => conclusion === 'failure')
		const logsUrl = failedStep ? `${job.html_url}#step:${failedStep.number}:1` : job.html_url
		return { succeeded: false, line: `:x: ${label} build *FAILED* (<${logsUrl}|view logs>)` }
	}

	const sizeMiB = Math.round(artifact.size_in_bytes / 1024 / 1024)
	const downloadUrl = `${runUrl}/artifacts/${artifact.id}`
	return {
		succeeded: true,
		line: `:white_check_mark: ${label} build *SUCCESS* (<${downloadUrl}|download> ${sizeMiB}MiB)`,
	}
}

const [{ jobs }, { artifacts }] = await Promise.all([
	fetchRunResource('jobs'),
	fetchRunResource('artifacts'),
])

const builds = PLATFORMS.map((platform) => summarizeBuild(platform, jobs, artifacts))
const statusEmoji = builds.every(({ succeeded }) => succeeded) ? ':white_check_mark:' : ':x:'
const headline = `A new Modrinth app build has been completed ${statusEmoji}`

const now = new Date()
const timestamp = `<!date^${Math.floor(now.getTime() / 1000)}^{date_short_pretty} at {time}|${now.toUTCString()}>`

const response = await fetch(SLACK_WEBHOOK_URL, {
	method: 'POST',
	headers: { 'Content-Type': 'application/json' },
	body: JSON.stringify({
		text: headline,
		blocks: [
			mrkdwnSection(`:modrinth: ${headline}`),
			mrkdwnContext(
				[
					`Environment: \`${BUILD_ENVIRONMENT}\``,
					`Branch: \`${escapeMrkdwn(GITHUB_REF_NAME)}\``,
					`Commit: \`${GITHUB_SHA.slice(0, 8)}\``,
					`Manually run by <${GITHUB_SERVER_URL}/${GITHUB_TRIGGERING_ACTOR}|${GITHUB_TRIGGERING_ACTOR}>`,
				].join('\n'),
			),
			mrkdwnSection(builds.map(({ line }) => line).join('\n')),
			{ type: 'divider' },
			mrkdwnContext(`<${runUrl}|View on GitHub> • ${timestamp}`),
		],
	}),
})

if (!response.ok) {
	throw new Error(`Failed to post to Slack: ${response.status} ${await response.text()}`)
}
