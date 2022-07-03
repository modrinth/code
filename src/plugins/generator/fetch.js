import { fetch as baseFetch } from 'undici'
import { promises as fs } from 'fs'

const API_URL =
	process.env.VITE_API_URL && process.env.VITE_API_URL === 'https://staging-api.modrinth.com/v2/'
		? 'https://staging-api.modrinth.com/v2/'
		: 'https://api.modrinth.com/v2/'

let version = ''

export async function fetch(route, options = {}) {
	if (!version) {
		version = JSON.parse(await fs.readFile('./package.json', 'utf8')).version
	}
	return baseFetch(API_URL + route, {
		...options,
		headers: {
			'user-agent': `Omorphia / ${version} (venashial@modrinth.com)`,
		},
	})
}
