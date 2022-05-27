import { fetch } from 'undici'
import { promises as fs, createWriteStream } from 'fs'
import cliProgress from 'cli-progress'
import Jimp from 'jimp'
import { getAverageColor } from 'fast-average-color-node'

// Note: This function has issues and will occasionally fail with some project icons. It averages at a 99.4% success rate. Most issues are from ECONNRESET errors & Jimp not being able to handle webp & svg images.
export async function projectColors(API_URL) {
	const progressBar = new cliProgress.SingleBar({
		format: 'Generating project colors | {bar} | {percentage}% || {value}/{total} projects',
		barCompleteChar: '\u2588',
		barIncompleteChar: '\u2591',
		hideCursor: true,
	})
	// Get total number of projects
	const projectCount = (await (await fetch(API_URL + 'search?limit=0')).json()).total_hits
	progressBar.start(projectCount, 0)
	const writeStream = createWriteStream('./generated/projects.json')
	writeStream.write('{')
	// Used to form the JSON string (so that the first doesn't have a comma prefix)
	let first = true
	let completed = 0
	// Number of pages through search to fetch
	const requestCount = Math.ceil(projectCount / 100)
	await Promise.allSettled(
		Array.from({ length: requestCount }, async (_, index) => {
			const response = await fetch(API_URL + `search?limit=100&offset=${index * 100}`)
			if (!response.ok) {
				throw new Error(`Failed to fetch projects: ${response.statusText}`)
			}
			// Get project hits & use map to get rid of extra data
			const hits = (await response.json()).hits.map((project) => ({
				project_id: project.project_id,
				slug: project.slug,
				title: project.title,
				icon_url: project.icon_url,
			}))
			// Try parsing the icon of each project
			await Promise.allSettled(
				hits.map(async (project) => {
					if (
						project.icon_url &&
						// Jimp doesn't support webp or svg
						!project.icon_url.endsWith('.webp') &&
						!project.icon_url.endsWith('.svg')
					) {
						try {
							const image = await Jimp.read(
								project.icon_url.replace('cdn', 'cdn-raw') // Skip redirect to raw CDN
							)
							// Resize image before getting average color (faster)
							image.resize(256, 256)
							// Get bottom edge of image
							const edge = image.clone().crop(0, 255, 256, 1)
							const buffer = await edge.getBufferAsync(Jimp.AUTO)
							let color = (await getAverageColor(buffer)).hexa
							// If the edge is transparent, use the average color of the entire image
							if (color === '#00000000') {
								const buffer = await image.getBufferAsync(Jimp.AUTO)
								color = (await getAverageColor(buffer)).hexa
							}
							// Remove color transparency
							color = color.replace(/.{2}$/, '')
							// Only use comma prefix if not first
							let prefix = ','
							if (first) {
								prefix = ''
								first = false
							}
							writeStream.write(`${prefix}"${project.project_id}":"${color}"`)
							completed++
						} catch (error) {
							// Ignore errors
							// console.log(error);
						}
					}
					progressBar.increment()
				})
			)
		})
	)
	writeStream.write('}')
	writeStream.end()
	progressBar.stop()
	console.log(`Failed to parse ${projectCount - completed} project icons.`)
}
