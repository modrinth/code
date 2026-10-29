/* eslint-disable @typescript-eslint/no-explicit-any */
import 'ace-builds/src-noconflict/mode-rust'

import ace from 'ace-builds'
import type { Ace } from 'ace-builds'

const RHAI_KEYWORDS =
	'let const fn private if else switch for in while loop do until break continue return throw try catch import export as is this'

const RHAI_SNIPPETS = [
	{ caption: 'let', snippet: 'let ${1:name} = ${2:value};' },
	{ caption: 'const', snippet: 'const ${1:NAME} = ${2:value};' },
	{ caption: 'fn', snippet: 'fn ${1:name}(${2:args}) {\n\t${0}\n}' },
	{ caption: 'if', snippet: 'if ${1:condition} {\n\t${0}\n}' },
	{ caption: 'for', snippet: 'for ${1:item} in ${2:items} {\n\t${0}\n}' },
	{ caption: 'while', snippet: 'while ${1:condition} {\n\t${0}\n}' },
	{ caption: 'try', snippet: 'try {\n\t${1}\n} catch (${2:error}) {\n\t${0}\n}' },
]

const RHAI_COMPLETER: Ace.Completer = {
	getCompletions(_editor, _session, _position, _prefix, callback) {
		callback(null, [
			...`${RHAI_KEYWORDS} true false`.split(' ').map((value) => ({
				caption: value,
				value,
				meta: 'Rhai keyword',
				score: 100,
			})),
			...RHAI_SNIPPETS.map((snippet) => ({ ...snippet, meta: 'Rhai snippet', score: 90 })),
		])
	},
}

export const RHAI_EDITOR_OPTIONS: Partial<Ace.EditorOptions> = {
	useWorker: false,
	tabSize: 4,
	useSoftTabs: false,
	enableBasicAutocompletion: [RHAI_COMPLETER],
	enableLiveAutocompletion: [RHAI_COMPLETER],
	enableSnippets: true,
}

ace['define'](
	'ace/mode/rhai_highlight_rules',
	['require', 'exports', 'ace/lib/oop', 'ace/mode/text_highlight_rules'],
	function (require, exports) {
		const oop = require('ace/lib/oop')
		const TextHighlightRules = require('ace/mode/text_highlight_rules').TextHighlightRules

		const RhaiHighlightRules = function (this: any) {
			const keywords = this.createKeywordMapper(
				{
					keyword: RHAI_KEYWORDS.replaceAll(' ', '|'),
					'constant.language': 'true|false',
				},
				'identifier',
				false,
			)

			this.$rules = {
				start: [
					{ token: 'comment', regex: /\/\/.*$/.source },
					{ token: 'comment', regex: /\/\*/.source, push: 'comment' },
					{ token: 'string', regex: /"/.source, push: 'string' },
					{ token: 'string', regex: /`/.source, push: 'template' },
					{
						token: 'string',
						regex: /'(?:\\(?:u\{[\da-fA-F]+\}|u[\da-fA-F]{4}|x[\da-fA-F]{2}|.)|[^'\\])'/.source,
					},
					{ token: 'constant.language', regex: /\(\s*\)/.source },
					{
						token: 'constant.numeric',
						regex:
							/\b(?:0x[\da-fA-F][\da-fA-F_]*|0o[0-7][0-7_]*|0b[01][01_]*|\d[\d_]*(?:\.(?!\.)\d[\d_]*)?(?:[eE][+-]?\d[\d_]*)?)\b/
								.source,
					},
					{
						token: (value: string) => {
							const token = keywords(value)
							return token === 'identifier' ? 'entity.name.function' : token
						},
						regex: /[a-zA-Z_][\w]*(?=\s*\()/.source,
					},
					{ token: keywords, regex: /[a-zA-Z_][\w]*/.source },
					{ token: 'paren.lparen', regex: /#\{|[\[({]/.source },
					{ token: 'paren.rparen', regex: /[\])}]/.source },
					{
						token: 'keyword.operator',
						regex:
							/\*\*=|<<=|>>=|\.\.=|==|!=|<=|>=|&&|\|\||\?\?|\?\.|=>|::|\*\*|<<|>>|\.\.|[+\-*/%&|^]=?|[!~<>=?:.]/
								.source,
					},
					{ token: 'punctuation.operator', regex: /[,;]/.source },
					{ token: 'text', regex: /\s+/.source },
				],
				comment: [
					{ token: 'comment', regex: /\/\*/.source, push: 'comment' },
					{ token: 'comment', regex: /\*\//.source, next: 'pop' },
					{ defaultToken: 'comment' },
				],
				string: [
					{ token: 'constant.language.escape', regex: /\\./.source },
					{ token: 'string', regex: /"/.source, next: 'pop' },
					{ defaultToken: 'string' },
				],
				template: [
					{ token: 'constant.language.escape', regex: /\\./.source },
					{ token: 'paren.lparen', regex: /\$\{/.source, push: 'interpolation' },
					{ token: 'string', regex: /`/.source, next: 'pop' },
					{ defaultToken: 'string' },
				],
				interpolation: [
					{ token: 'paren.lparen', regex: /#?\{/.source, push: 'interpolation' },
					{ token: 'paren.rparen', regex: /\}/.source, next: 'pop' },
					{ include: 'start' },
				],
			}
			this.normalizeRules()
		}

		oop.inherits(RhaiHighlightRules, TextHighlightRules)
		exports.RhaiHighlightRules = RhaiHighlightRules
	},
)

ace['define'](
	'ace/mode/rhai',
	[
		'require',
		'exports',
		'ace/lib/oop',
		'ace/mode/rust',
		'ace/mode/rhai_highlight_rules',
		'ace/mode/folding/cstyle',
	],
	function (require, exports) {
		const oop = require('ace/lib/oop')
		const RustMode = require('ace/mode/rust').Mode
		const RhaiHighlightRules = require('ace/mode/rhai_highlight_rules').RhaiHighlightRules
		const FoldMode = require('ace/mode/folding/cstyle').FoldMode

		const Mode = function (this: any) {
			RustMode.call(this)
			this.HighlightRules = RhaiHighlightRules
			this.foldingRules = new FoldMode()
			this.lineCommentStart = '//'
			this.blockComment = { start: '/*', end: '*/', nestable: true }
			this.$id = 'ace/mode/rhai'
		}

		oop.inherits(Mode, RustMode)
		exports.Mode = Mode
	},
)
