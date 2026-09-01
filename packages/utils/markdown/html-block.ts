import { defineComarkPlugin } from 'comark/parse'
import { Parser as HtmlTagParser } from 'htmlparser2'

// This is here to keep backwards compatibility
// Comark closes most open html tags when it hits a blank line,
// we make it only do that if we can't find a closing tag, in order to mimic the side effect of
// markdown-it emitting html entirely raw (which your browser is then lenient with unclosed tags)
// whereas comark does its fancy AST which means it can't just emit the html raw
// Thank you for coming to my TED talk -chyz


// If you know a better way to do this go ahead
function findHtmlBlockRuleFn(md: any): any | undefined {
	const rules = (md.block.ruler as unknown as { __rules__: { name: string; fn: any }[] })
		.__rules__
	return rules?.find((r) => r.name === 'comark_html_block')?.fn
}

function extractOpenTagName(line: string): string | undefined {
	return /^<([a-zA-Z][a-zA-Z0-9-]*)/.exec(line)?.[1]?.toLowerCase()
}

function parseOpenTagAttrs(line: string): [string, string][] {
	const attrs: [string, string][] = []
	const parser = new HtmlTagParser({
		onopentag(_name, attrObj) {
			for (const [key, value] of Object.entries(attrObj)) attrs.push([key, value])
		},
	})
	parser.write(line)
	parser.end()
	return attrs
}

function htmlBlockCompatRule(md: any) {
	let htmlBlockFn: any

	md.block.ruler.before(
		'html_block',
		'html_block_compat',
		(state: any, startLine: number, endLine: number, silent: boolean) => {
			if (!htmlBlockFn) {
				htmlBlockFn = findHtmlBlockRuleFn(md)
				if (!htmlBlockFn) return false
			}
			if (silent) return htmlBlockFn(state, startLine, endLine, true)

			const tokensBefore = state.tokens.length
			const lineBefore = state.line
			const matched = htmlBlockFn(state, startLine, endLine, false)
			if (!matched) return false

			const token = state.tokens[state.tokens.length - 1]
			if (token.type !== 'html_block' || !token.map) return true

			const firstLine = (token.content.split('\n')[0] ?? '').trim()
			if (firstLine.endsWith('/>')) return true

			const tagName = extractOpenTagName(firstLine)
			if (!tagName) return true

			const openLineRe = new RegExp(`^<${tagName}(\\s|/?>|$)`, 'i')
			const closeLineRe = new RegExp(`^</${tagName}\\s*>$`, 'i')
			let depth = 0
			let closeLine = -1
			for (let line = startLine + 1; line < endLine; line++) {
				if (state.sCount[line] < state.blkIndent) break
				const p = state.bMarks[line] + state.tShift[line]
				const mx = state.eMarks[line]
				const text = state.src.slice(p, mx).trim()
				if (openLineRe.test(text) && !text.endsWith('/>')) {
					depth++
					continue
				}
				if (closeLineRe.test(text)) {
					if (depth > 0) {
						depth--
						continue
					}
					closeLine = line
					break
				}
			}
			if (closeLine === -1 || closeLine < token.map[1]) return true

			state.tokens.length = tokensBefore
			state.line = lineBefore

			const attrs = parseOpenTagAttrs(firstLine)
			const oldParent = state.parentType
			const oldLineMax = state.lineMax
			state.parentType = 'comark_block' as any
			state.lineMax = closeLine

			const tokenOpen: any = state.push('mdc_block_open', tagName, 1)
			tokenOpen.block = true
			tokenOpen.map = [startLine, closeLine + 1]
			for (const [key, value] of attrs) tokenOpen.attrSet(key, value)

			const blkIndent = state.blkIndent
			state.blkIndent = 0
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const env = state.env as any
			env.comarkBlockTokens ||= []
			env.comarkBlockTokens.unshift(tokenOpen)
			state.md.block.tokenize(state, startLine + 1, closeLine)
			state.blkIndent = blkIndent
			env.comarkBlockTokens.shift()

			const tokenClose: any = state.push('mdc_block_close', tagName, -1)
			tokenClose.map = [startLine, closeLine + 1]
			tokenClose.block = true

			state.parentType = oldParent
			state.lineMax = oldLineMax
			state.line = closeLine + 1
			return true
		},
		{ alt: ['paragraph', 'reference', 'blockquote'] },
	)
}

export const htmlBlock = defineComarkPlugin(() => ({
	name: 'html-block-compat',
	markdownItPlugins: [htmlBlockCompatRule],
}))
