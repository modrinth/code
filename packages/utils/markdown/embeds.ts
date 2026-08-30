import { defineComarkPlugin } from 'comark/parse'

const embedTypes: Record<string, string[]> = {
	project: ['project', 'mod', 'modpack', 'resourcepack', 'shader', 'plugin', 'datapack'],
	user: ['user'],
	organization: ['organization', 'org'],
	collection: ['collection'],
}

const embedShorthandPrefixes: Record<string, string> = {
	'@': 'user',
}

const embedTypeAliases: Record<string, string> = Object.fromEntries(
	Object.entries(embedTypes).flatMap(([tag, aliases]) => aliases.map((alias) => [alias, tag])),
)

const aliasPattern = Object.keys(embedTypeAliases)
	.sort((a, b) => b.length - a.length)
	.join('|')
const embedIdPattern = /[\w.-]+/.source
const shorthandCharClass = Object.keys(embedShorthandPrefixes)
	.map((char) => char.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
	.join('')

const embedSyntaxRegex = new RegExp(
	`<(?:(${aliasPattern})/(${embedIdPattern})|([${shorthandCharClass}])(${embedIdPattern}))>`,
	'y',
)

const embedRule = (md: any) => {
	md.block.ruler.before(
		'paragraph',
		'embed',
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		(state: any, startLine: number, _endLine: number, silent: boolean) => {
			const start = state.bMarks[startLine] + state.tShift[startLine]
			const max = state.eMarks[startLine]
			if (state.src.charCodeAt(start) !== 0x3c) return false

			embedSyntaxRegex.lastIndex = start
			const match = embedSyntaxRegex.exec(state.src)
			if (!match || match.index !== start || state.src.slice(start, max).trim() !== match[0]) return false

			if (silent) return true

			const tag = match[1] ? embedTypeAliases[match[1]] : embedShorthandPrefixes[match[3]]
			const id = match[2] ?? match[4]

			const tokenOpen = state.push('mdc_block_open', tag, 1)
			tokenOpen.block = true
			tokenOpen.map = [startLine, startLine + 1]
			tokenOpen.attrSet('id', id)
			state.push('mdc_block_close', tag, -1)

			state.line = startLine + 1
			return true
		},
		{ alt: ['paragraph', 'reference', 'blockquote', 'list'] },
	)
}

export const embedSyntax = defineComarkPlugin(() => ({
	name: 'embed-syntax',
	markdownItPlugins: [embedRule],
}))
