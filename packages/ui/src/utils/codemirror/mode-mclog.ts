import type { StreamParser } from '@codemirror/language'

const TIMESTAMP = /^\[\d\d:\d\d:\d\d\]/

/** The parts of a log line after its timestamp, by the token each is shown as, tried in order. */
const RULES: [RegExp, string][] = [
	[/^\[[^\]]+?\/ERROR\]:?/, 'invalid'],
	[/^\[[^\]]+?\/WARN\]:?/, 'keyword'],
	[/^\[[^\]]+?\/INFO\]:/, 'string'],
	[/^: \/.+/, 'meta'],
	[/^\tat\s.+/, 'comment'],
	[/^\w+?\[\/\d+?\.\d+?\.\d+?\.\d+?:\d+?\]/, 'variableName.function'],
	[/^\[CHAT\]/, 'atom'],
]

/** Minecraft logs (`latest.log`), with their timestamps, levels, commands and stack traces. */
export const mclog: StreamParser<null> = {
	name: 'mclog',
	token(stream) {
		if (stream.sol() && stream.match(TIMESTAMP)) return 'comment'
		for (const [pattern, token] of RULES) {
			if (stream.match(pattern)) return token
		}
		if (stream.eatWhile(/\w/)) return null
		stream.next()
		stream.eatWhile(/[^[:\t\w]/)
		return null
	},
}
