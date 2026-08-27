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

const markdownItModrinthEmbed = (md: any) => {
	md.inline.ruler.before(
		'html_inline',
		'modrinth-embed',
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		(state: any, silent: boolean) => {
			if (state.src.charCodeAt(state.pos) !== 0x3c) return false

			embedSyntaxRegex.lastIndex = state.pos
			const match = embedSyntaxRegex.exec(state.src)
			if (!match) return false

			const tag = match[1] ? embedTypeAliases[match[1]] : embedShorthandPrefixes[match[3]]
			const id = match[2] ?? match[4]

			if (!silent) {
				state.push('mdc_inline_component', tag, 0)
				const propsToken = state.push('mdc_inline_props', 'span', 0)
				propsToken.attrs = [['id', id]]
				propsToken.hidden = true
			}

			state.pos = embedSyntaxRegex.lastIndex
			return true
		},
	)
}

export const modrinthEmbedSyntax = defineComarkPlugin(() => ({
	name: 'modrinth-embed-syntax',
	markdownItPlugins: [markdownItModrinthEmbed],
}))
