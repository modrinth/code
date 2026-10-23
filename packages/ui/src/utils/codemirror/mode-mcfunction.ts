import type { StreamParser } from '@codemirror/language'

/** The parts of a command after its name, by the token each is shown as, tried in order. */
const RULES: [RegExp, string][] = [
	[/^"(?:\\.|[^"\\])*"/, 'string'],
	[/^'(?:\\.|[^'\\])*'/, 'string'],
	[/^\$\([a-zA-Z0-9_]+\)/, 'variableName'],
	[/^@[apers]\b/i, 'atom'],
	[/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}\b/i, 'atom'],
	[/^#?[a-z0-9_.]+:[a-z0-9_./-]+/i, 'typeName'],
	[/^run\s+\/?[a-z][a-z0-9_]*/i, 'keyword'],
	[/^[~^]-?(?:\d*\.)?\d+/, 'number'],
	[/^[~^]/, 'operator'],
	[/^-?(?:\d*\.)?\d+[bdfils]?\b/i, 'number'],
	[/^(?:\.\.|(?:true|false)\b)/i, 'atom'],
]

/** Minecraft function files (`.mcfunction`) and command history, one command per line. */
export const mcfunction: StreamParser<null> = {
	name: 'mcfunction',
	token(stream) {
		if (stream.sol()) {
			stream.eatSpace()
			if (stream.match(/^#(?![a-z0-9_.]+:)/i)) {
				stream.skipToEnd()
				return 'comment'
			}
			if (stream.match(/^\/?[a-z][a-z0-9_]*/i)) return 'keyword'
		}
		if (stream.eatSpace()) return null
		for (const [pattern, token] of RULES) {
			if (stream.match(pattern)) return token
		}
		if (!stream.eatWhile(/\w/)) stream.next()
		return null
	},
	languageData: { commentTokens: { line: '#' } },
}
