interface FencedBlockConfig {
	marker: string
	ruleName: string
	ruleBefore: string
	minLength?: number
	parentType: string
	suffixChar?: string
	validate?(openText: string, hasSuffix: boolean): boolean
	pushOpen(state: any, openText: string, hasSuffix: boolean, startLine: number, closeLine: number): void
	pushClose(state: any): void
}

export function fencedBlockRule(config: FencedBlockConfig) {
	const { marker, ruleName, ruleBefore, minLength = 3, parentType, suffixChar, validate, pushOpen, pushClose } =
		config

	return (md: any) => {
		md.block.ruler.before(
			ruleBefore,
			ruleName,
			(state: any, startLine: number, endLine: number, silent: boolean) => {
				const start = state.bMarks[startLine] + state.tShift[startLine]
				const max = state.eMarks[startLine]
				const indent = state.sCount[startLine]

				if (state.src[start] !== marker) return false

				let pos = start
				while (pos < max && state.src[pos] === marker) pos++
				const markerLen = pos - start

				const hasSuffix = suffixChar !== undefined && state.src[pos] === suffixChar
				if (markerLen + (hasSuffix ? 1 : 0) < minLength) return false
				if (hasSuffix) pos++

				const openText = state.src.slice(pos, max).trim()
				if (validate && !validate(openText, hasSuffix)) return false

				if (silent) return true

				let closeLine = -1
				for (let line = startLine + 1; line < endLine; line++) {
					const lineStart = state.bMarks[line] + state.tShift[line]
					const lineEnd = state.eMarks[line]
					if (lineStart < lineEnd && state.sCount[line] < indent) break
					if (state.sCount[line] !== indent || state.src[lineStart] !== marker) continue

					let closePos = lineStart
					while (closePos < lineEnd && state.src[closePos] === marker) closePos++
					if (closePos - lineStart < markerLen) continue

					const closeHasSuffix = suffixChar !== undefined && state.src[closePos] === suffixChar
					if (closeHasSuffix !== hasSuffix) continue
					if (closeHasSuffix) closePos++

					if (state.src.slice(closePos, lineEnd).trim() === '') {
						closeLine = line
						break
					}
				}
				if (closeLine === -1) closeLine = endLine

				const oldParent = state.parentType
				const oldLineMax = state.lineMax
				const oldIndent = state.blkIndent
				state.parentType = parentType
				state.lineMax = closeLine
				state.blkIndent = indent

				pushOpen(state, openText, hasSuffix, startLine, closeLine)
				state.md.block.tokenize(state, startLine + 1, closeLine)
				pushClose(state)

				state.parentType = oldParent
				state.lineMax = oldLineMax
				state.blkIndent = oldIndent
				state.line = closeLine < endLine ? closeLine + 1 : closeLine
				return true
			},
			{ alt: ['paragraph', 'reference', 'blockquote', 'list'] },
		)
	}
}
