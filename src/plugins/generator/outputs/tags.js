import { fetch } from '../fetch.js'
import { promises as fs } from 'fs'
import cliProgress from 'cli-progress'

export async function tags() {
	const progressBar = new cliProgress.SingleBar({
		format: 'Generating tags           | {bar} | {percentage}%',
		barCompleteChar: '\u2588',
		barIncompleteChar: '\u2591',
		hideCursor: true,
	})
	progressBar.start(7, 0)

	// eslint-disable-next-line prefer-const
	let [categories, loaders, licenses, donationPlatforms, reportTypes] = await Promise.all([
		await (await fetch('tag/category')).json(),
		await (await fetch('tag/loader')).json(),
		await (await fetch('tag/license')).json(),
		await (await fetch('tag/donation_platform')).json(),
		await (await fetch('tag/report_type')).json(),
	])
	progressBar.update(5)

	const tagIconReducer = (a, v) => ({
		...a,
		[v.name]: v.icon.replace('<svg', '<svg class="icon"'),
	})

	// Create single object with icons
	const tagIcons = {
		...categories.reduce(tagIconReducer, {}),
		...loaders.reduce(tagIconReducer, {}),
	}
	progressBar.increment()

	// Delete icons from original arrays
	categories = categories.map(({ icon, ...rest }) => rest)
	loaders = loaders.map(({ icon, ...rest }) => rest)
	progressBar.increment()

	// Set project types
	const projectTypes = ['mod', 'modpack']

	// Write JSON file
	await fs.writeFile(
		'./generated/tags.json',
		JSON.stringify({
			categories,
			loaders,
			projectTypes,
			licenses,
			donationPlatforms,
			reportTypes,
			tagIcons,
		})
	)
	progressBar.increment()

	progressBar.stop()
}
