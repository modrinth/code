import { fetch } from '../fetch.js'
import { promises as fs } from 'fs'
import cliProgress from 'cli-progress'

export async function gameVersions() {
	const progressBar = new cliProgress.SingleBar({
		format: 'Generating game versions  | {bar} | {percentage}%',
		barCompleteChar: '\u2588',
		barIncompleteChar: '\u2591',
		hideCursor: true,
	})
	progressBar.start(2, 0)

	const gameVersions = await (await fetch('tag/game_version')).json()
	progressBar.increment()

	// Write JSON file
	await fs.writeFile('./generated/gameVersions.json', JSON.stringify(gameVersions))
	progressBar.increment()

	progressBar.stop()
}
