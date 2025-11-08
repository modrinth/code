import type { HLJSApi } from 'highlight.js'

/*
Language: Skript
Description: Skript language support for Minecraft server scripting
Website: https:
Category: scripting
*/
export default function (hljs: HLJSApi) {
	const CONTROL_KEYWORDS = {
		keyword: 'if else while loop return continue at stop cancel false true now',
		built_in: 'parse do',
	}

	const ENTITIES = 'player players victim attacker sender loop-player shooter console'
	const CONDITION_OPERATORS =
		'contains has have is was were are does can cannot ' +
		"hasn't haven't isn't wasn't weren't aren't doesn't can't"

	return {
		name: 'Skript',
		aliases: ['sk'],
		case_insensitive: true,
		keywords: CONTROL_KEYWORDS,
		contains: [
			{
				className: 'comment',
				begin: '(?<!#)#(?!#)',
				end: '$',
				relevance: 0,
			},
			{
				className: 'string',
				begin: '"',
				end: '"',
				illegal: '\\n',
				contains: [hljs.BACKSLASH_ESCAPE],
			},

			{
				className: 'section',
				begin: '\\bon\\s+',
				end: ':',
				excludeEnd: false,
				keywords: 'on',
				relevance: 10,
			},

			{
				className: 'section',
				begin: '\\bcommand\\s+/',
				end: ':',
				excludeEnd: false,
				relevance: 10,
			},

			{
				className: 'variable',
				begin: '{',
				end: '}',
				contains: [
					{
						className: 'variable',
						begin: ':+',
						relevance: 0,
					},
				],
			},

			{
				className: 'params',
				begin: '<',
				end: '>',
				relevance: 5,
			},

			{
				className: 'function',
				begin: '\\b[a-zA-Z_][a-zA-Z0-9_]*(?=\\()',
				relevance: 0,
			},

			{
				className: 'variable',
				begin: '\\b(loop|event)-[a-zA-Z]+\\b',
				relevance: 5,
			},

			{
				className: 'number',
				variants: [
					{ begin: '\\b\\d+(\\.\\d+)?\\s+(tick|second|minute|hour|day)s?\\b' },

					{ begin: '\\ba\\s+(tick|second|minute|hour|day)s?\\b' },

					{ begin: '\\b(minecraft|mc|real|rl|irl)\\s+(tick|second|minute|hour|day)s?\\b' },
				],
				relevance: 0,
			},

			hljs.NUMBER_MODE,

			{
				className: 'built_in',
				begin: '\\b(' + ENTITIES + ')\\b',
				relevance: 0,
			},

			{
				className: 'built_in',
				begin: "(uuid\\s+of|'s\\s+uuid|location\\s+of|'s\\s+location)",
				relevance: 0,
			},

			{
				className: 'operator',
				begin: '\\b(' + CONDITION_OPERATORS.split(' ').join('|') + ')\\b',
				relevance: 0,
			},

			{
				className: 'operator',
				begin: '::?',
				relevance: 0,
			},

			{
				className: 'literal',
				begin: '\\b(true|false)\\b',
				relevance: 0,
			},

			{
				className: 'keyword',
				begin: '\\b(stop|cancel|halt|enable|disable|trigger|server)\\b',
				relevance: 5,
			},
		],
	}
}
