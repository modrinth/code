import { ComponentParser } from 'sveld'
import * as svelte from 'svelte/compiler'
import fs from 'fs/promises'
import path from 'path'
import { preprocess } from '../../src/config/svelte.js'

export default function sveld() {
	return {
		name: 'vite-plugin-sveld',
		// This generates a `COMPONENT_API.json` with sveld in the `/_app` folder which is used by the docs about components
		// TODO: Make more efficient & handle typescript types with `svelte2tsx`
		async transform(src, id) {
			if (id.includes('/src/components/')) {
				const output = {}

				const componentFiles = await fs.readdir(path.resolve('./src/components'))

				for (const fileName of componentFiles.filter((name) => name.endsWith('.svelte'))) {
					const filePath = path.resolve('./src/components', fileName)
					const raw = (await fs.readFile(filePath)).toString()
					output[fileName] = await parseRaw(raw, filePath)
				}

				try {
					await fs.mkdir(path.resolve('./generated'))
				} catch {
					// Do nothing, directory already exists
				}

				await fs.writeFile(path.resolve('./generated/COMPONENT_API.json'), JSON.stringify(output))
			}
		},
	}
}

async function parseRaw(raw, filePath) {
	let { code } = await svelte.preprocess(raw, preprocess, {
		filename: filePath,
	})
	return new ComponentParser({
		verbose: false,
	}).parseSvelteComponent(code, {
		filePath,
		moduleName: filePath,
	})
}
