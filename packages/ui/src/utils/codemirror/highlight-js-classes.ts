import { tagHighlighter, tags as t } from '@lezer/highlight'

/**
 * Gives each token the class highlight.js would give it, so editors are coloured by the same theme
 * as highlighted code blocks (see `useCodeHighlightTheme`), with no colours of their own.
 */
export const highlightJsClasses = tagHighlighter([
	{ tag: [t.comment, t.lineComment, t.blockComment, t.docComment], class: 'hljs-comment' },
	{
		tag: [
			t.keyword,
			t.controlKeyword,
			t.definitionKeyword,
			t.moduleKeyword,
			t.operatorKeyword,
			t.modifier,
		],
		class: 'hljs-keyword',
	},
	{ tag: [t.self], class: 'hljs-variable language_' },
	{ tag: [t.string, t.docString, t.character, t.attributeValue], class: 'hljs-string' },
	{ tag: [t.special(t.string), t.url], class: 'hljs-link' },
	{ tag: [t.regexp], class: 'hljs-regexp' },
	{ tag: [t.escape], class: 'hljs-char escape_' },
	{ tag: [t.number, t.integer, t.float], class: 'hljs-number' },
	{ tag: [t.bool, t.null, t.atom], class: 'hljs-literal' },
	{ tag: [t.typeName, t.standard(t.typeName)], class: 'hljs-type' },
	{ tag: [t.className, t.namespace], class: 'hljs-title class_' },
	{
		tag: [t.function(t.variableName), t.function(t.propertyName), t.macroName],
		class: 'hljs-title function_',
	},
	{ tag: [t.standard(t.variableName)], class: 'hljs-built_in' },
	{ tag: [t.variableName, t.special(t.variableName)], class: 'hljs-variable' },
	{ tag: [t.propertyName, t.attributeName], class: 'hljs-attr' },
	{ tag: [t.labelName], class: 'hljs-symbol' },
	{ tag: [t.tagName], class: 'hljs-name' },
	{ tag: [t.meta, t.annotation, t.processingInstruction], class: 'hljs-meta' },
	{ tag: [t.operator], class: 'hljs-operator' },
	{ tag: [t.punctuation], class: 'hljs-punctuation' },
	{ tag: [t.heading], class: 'hljs-section' },
	{ tag: [t.strong], class: 'hljs-strong' },
	{ tag: [t.emphasis], class: 'hljs-emphasis' },
	{ tag: [t.link], class: 'hljs-link' },
	{ tag: [t.quote], class: 'hljs-quote' },
	{ tag: [t.list], class: 'hljs-bullet' },
	{ tag: [t.monospace], class: 'hljs-code' },
	{ tag: [t.inserted], class: 'hljs-addition' },
	{ tag: [t.deleted, t.invalid], class: 'hljs-deletion' },
])
