/* eslint-disable @typescript-eslint/no-explicit-any */
import ace from 'ace-builds'

ace['define'](
	'ace/mode/mcfunction_highlight_rules',
	['require', 'exports', 'ace/lib/oop', 'ace/mode/text_highlight_rules'],
	function (require: any, exports: any) {
		const oop = require('ace/lib/oop')
		const TextHighlightRules = require('ace/mode/text_highlight_rules').TextHighlightRules

		const McfunctionHighlightRules = function (this: any) {
			this.$rules = {
				start: [
					{
						token: 'comment.doc',
						regex: /^\s*#>.*$/.source,
					},
					{
						token: 'comment',
						regex: /^\s*#!.*$/.source,
					},
					{
						token: 'comment',
						regex: /^\s*##.*$/.source,
					},
					{
						token: 'comment',
						regex: /^\s*#(?![a-z0-9_.]+:).*$/i.source,
					},
					{
						token: 'string',
						regex: /"(?:\\.|[^"\\])*"/.source,
					},
					{
						token: 'string',
						regex: /'(?:\\.|[^'\\])*'/.source,
					},
					{
						token: 'variable',
						regex: /\$\([a-zA-Z0-9_]+\)/.source,
					},
					{
						token: 'constant.language',
						regex: /@[apers]\b/i.source,
					},
					{
						token: 'constant.language',
						regex: /\b[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}\b/i.source,
					},
					{
						token: 'support.constant',
						regex: /#?[a-z0-9_.]+:[a-z0-9_./-]+/i.source,
					},
					{
						token: 'keyword',
						regex: /\brun\s+\/?[a-z][a-z0-9_]*/i.source,
					},
					{
						token: 'constant.numeric',
						regex: /[~^]-?(?:\d*\.)?\d+/.source,
					},
					{
						token: 'keyword.operator',
						regex: /[~^]/.source,
					},
					{
						token: 'constant.numeric',
						regex: /-?(?:\d*\.)?\d+[bdfils]?\b/i.source,
					},
					{
						token: 'constant.language',
						regex: /\.\.|\b(?:true|false)\b/i.source,
					},
					{
						token: 'keyword',
						regex: /^\s*\/?[a-z][a-z0-9_]*/i.source,
					},
				],
			}
			this.normalizeRules()
		}

		oop.inherits(McfunctionHighlightRules, TextHighlightRules)
		exports.McfunctionHighlightRules = McfunctionHighlightRules
	},
)

ace['define'](
	'ace/mode/mcfunction',
	['require', 'exports', 'ace/lib/oop', 'ace/mode/text', 'ace/mode/mcfunction_highlight_rules'],
	function (require: any, exports: any) {
		const oop = require('ace/lib/oop')
		const TextMode = require('ace/mode/text').Mode
		const McfunctionHighlightRules =
			require('ace/mode/mcfunction_highlight_rules').McfunctionHighlightRules

		const Mode = function (this: any) {
			this.HighlightRules = McfunctionHighlightRules
			this.$id = 'ace/mode/mcfunction'
		}

		oop.inherits(Mode, TextMode)
		exports.Mode = Mode
	},
)
