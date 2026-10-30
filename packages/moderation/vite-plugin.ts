import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

export const moderationMessagesDirectory = fileURLToPath(
	new URL('./src/data/messages/checklist/messages', import.meta.url),
).replaceAll('\\', '/')

export default function moderationMessages() {
	return {
		name: 'modrinth-moderation-messages',
		enforce: 'pre' as const,
		config() {
			return {
				resolve: {
					alias: {
						'#messages': moderationMessagesDirectory,
					},
				},
			}
		},
		async load(this: { addWatchFile: (id: string) => void }, id: string) {
			if (!id.startsWith(`${moderationMessagesDirectory}/`) || !id.endsWith('.md')) return null

			this.addWatchFile(id)
			const content = await readFile(id, 'utf8')
			return {
				code: `export default ${JSON.stringify(content)}`,
				map: null,
			}
		},
	}
}
