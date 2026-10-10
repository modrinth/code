import type { ImageViewerEditorItem } from '@modrinth/ui'
import { configuredXss, md } from '@modrinth/utils'
import { hljs } from '@modrinth/utils/highlightjs/index'

export type IssueMessageBlock =
	| { type: 'markdown'; html: string }
	| { type: 'copy'; text: string }
	| { type: 'images'; images: ImageViewerEditorItem[] }

/** Parses the two supported message blocks without interpreting code examples as components. */
export function parseIssueMessage(message: string, highlighted = false): IssueMessageBlock[] {
	const markdown = md({
		highlight: (text: string, language: string) =>
			highlighted && language && hljs.getLanguage(language)
				? hljs.highlight(text, { language }).value
				: '',
	})
	const decode = (value: string) => markdown.utils.unescapeAll(value.replaceAll('\\', '&#92;'))
	markdown.block.ruler.before(
		'html_block',
		'issue_message_block',
		(state, start, end, silent) => {
			if (state.level !== 0 || state.sCount[start] - state.blkIndent >= 4) return false
			const line = state.src.slice(state.bMarks[start] + state.tShift[start], state.eMarks[start])
			const image = /^<image-viewer\s+src="([^"\n]*)"(?:\s+alt="([^"\n]*)")?\s*\/>\s*$/.exec(line)
			if (image) {
				if (silent) return true
				const token = state.push('issue_image', '', 0)
				token.meta = { src: image[1], alt: image[2] ?? '' }
				state.line = start + 1
				return true
			}
			if (line.trim() !== '<copy-code>') return false
			let closing = start + 1
			while (closing < end) {
				const line = state.src.slice(state.bMarks[closing], state.eMarks[closing]).trim()
				if (line === '</copy-code>') break
				closing++
			}
			if (closing === end) return false
			const content = state.src.slice(state.bMarks[start + 1], state.bMarks[closing])
			const pre = /^<pre>([\s\S]*)<\/pre>\r?\n$/.exec(content)
			if (!pre) return false
			if (silent) return true
			const token = state.push('issue_copy', '', 0)
			token.content = pre[1]
			state.line = closing + 1
			return true
		},
		{ alt: ['paragraph', 'reference', 'blockquote', 'list'] },
	)
	const environment = {}
	const tokens = markdown.parse(message, environment)
	const blocks: IssueMessageBlock[] = []
	let pending: typeof tokens = []
	function flush() {
		if (!pending.length) return
		blocks.push({
			type: 'markdown',
			html: configuredXss.process(markdown.renderer.render(pending, markdown.options, environment)),
		})
		pending = []
	}
	let imageIndex = 0
	for (const token of tokens) {
		if (token.type === 'issue_copy') {
			flush()
			blocks.push({ type: 'copy', text: decode(token.content) })
		} else if (token.type === 'issue_image') {
			flush()
			try {
				const url = new URL(decode(token.meta.src))
				if (!['https:', 'http:'].includes(url.protocol)) continue
			} catch {
				continue
			}
			const safeImage = configuredXss.process(
				`<img src="${token.meta.src}" alt="${token.meta.alt}">`,
			)
			const src = /\bsrc="([^"]+)"/.exec(safeImage)?.[1]
			if (!src) continue
			const alt = /\balt="([^"]*)"/.exec(safeImage)?.[1] ?? ''
			const image = {
				id: String(imageIndex++),
				src: decode(src),
				alt: decode(alt),
				title: decode(alt),
			}
			const previous = blocks[blocks.length - 1]
			if (previous?.type === 'images') previous.images.push(image)
			else blocks.push({ type: 'images', images: [image] })
		} else pending.push(token)
	}
	flush()
	return blocks
}
