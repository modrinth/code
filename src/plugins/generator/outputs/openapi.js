import { promises as fs } from 'fs'
import cliProgress from 'cli-progress'
import openapiTS from 'openapi-typescript'

export async function openapi() {
	const progressBar = new cliProgress.SingleBar({
		format: 'Generating openapi types  | {bar} | {percentage}%',
		barCompleteChar: '\u2588',
		barIncompleteChar: '\u2591',
		hideCursor: true,
	})
	progressBar.start(2, 0)

	const output = await openapiTS('https://docs.modrinth.com/redocusaurus/plugin-redoc-0.yaml')
	progressBar.increment()

	// Write JSON file
	await fs.writeFile('./generated/openapi.ts', output)
	progressBar.increment()

	progressBar.stop()
}
