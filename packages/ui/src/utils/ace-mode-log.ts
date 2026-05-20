/* eslint-disable @typescript-eslint/no-explicit-any */
import ace from 'ace-builds'

ace['define'](
	'ace/mode/mclog_highlight_rules',
	['require', 'exports', 'ace/lib/oop', 'ace/mode/text_highlight_rules'],
	function (require: any, exports: any) {
		const oop = require('ace/lib/oop')
		const TextHighlightRules = require('ace/mode/text_highlight_rules').TextHighlightRules

		const MclogHighlightRules = function (this: any) {
			this.$rules = {
				start: [
					{
						token: 'comment.timestamp',
						regex: /^\[\d\d:\d\d:\d\d\]/.source,
					},
					{
						token: 'invalid.error',
						regex: /\[.+?\/ERROR\]:?/.source,
					},
					{
						token: 'keyword.warn',
						regex: /\[.+?\/WARN\]:?/.source,
					},
					{
						token: 'string.info',
						regex: /\[.+?\/INFO\]:/.source,
					},
					{
						token: 'support.command',
						regex: /: \/.+/.source,
					},
					{
						token: 'comment.stacktrace',
						regex: /\tat\s.+/.source,
					},
					{
						token: 'entity.name.function',
						regex: /\w+?\[\/\d+?\.\d+?\.\d+?\.\d+?:\d+?\]/.source,
					},
					{
						token: 'storage.chat',
						regex: /\[CHAT\]/.source,
					},
				],
			}
			this.normalizeRules()
		}

		oop.inherits(MclogHighlightRules, TextHighlightRules)
		exports.MclogHighlightRules = MclogHighlightRules
	},
)

ace['define'](
	'ace/mode/mclog',
	['require', 'exports', 'ace/lib/oop', 'ace/mode/text', 'ace/mode/mclog_highlight_rules'],
	function (require: any, exports: any) {
		const oop = require('ace/lib/oop')
		const TextMode = require('ace/mode/text').Mode
		const MclogHighlightRules = require('ace/mode/mclog_highlight_rules').MclogHighlightRules

		const Mode = function (this: any) {
			this.HighlightRules = MclogHighlightRules
		}

		oop.inherits(Mode, TextMode)
		exports.Mode = Mode
	},
)
