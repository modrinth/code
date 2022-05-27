import { marked } from 'marked'
import hljs from 'highlight.js'
import insane from 'insane'

const renderer = new marked.Renderer()

renderer.image = (href, text) => {
	if (/^https?:\/\/(www\.)?youtube\.com\/watch\?v=[a-zA-Z0-9_]{11}$/.test(href)) {
		const id = href.substring(32, 43)
		return `<iframe src="https://www.youtube-nocookie.com/embed/${id}?&modestbranding=1&autoplay=0&rel=0" frameborder="0" allowfullscreen></iframe>`
	} else {
		return `<img src="${href}" alt="${text}" />`
	}
}

renderer.link = (href, title, text) => {
	if (href === null) {
		return text
	}
	let out = '<a href="' + href + '" rel="external nofollow"'
	if (title) {
		out += ' title="' + title + '"'
	}
	out += '>' + text + '</a>'
	return out
}

marked.setOptions({
	renderer,
	highlight: function (code, lang) {
		const language = hljs.getLanguage(lang) ? lang : 'plaintext'
		return hljs.highlight(code, { language }).value
	},
	langPrefix: 'hljs language-',
	headerPrefix: '',
	gfm: true,
	smartLists: true,
})

function sanitize(html: string): string {
	return insane(html, {
		allowedAttributes: {
			a: ['href', 'target', 'title', 'rel'],
			iframe: ['allowfullscreen', 'src', 'width', 'height'],
			img: ['src', 'width', 'height', 'alt'],
			h1: ['id'],
			h2: ['id'],
			h3: ['id'],
			h4: ['id'],
			h5: ['id'],
			h6: ['id'],
			code: ['class'],
			span: ['class'],
			input: ['type', 'checked', 'disabled'],
			font: ['color'],
		},
		allowedClasses: {},
		allowedSchemes: ['http', 'https', 'mailto'],
		allowedTags: [
			'a',
			'b',
			'blockquote',
			'br',
			'caption',
			'center',
			'code',
			'del',
			'details',
			'div',
			'em',
			'font',
			'h1',
			'h2',
			'h3',
			'h4',
			'h5',
			'h6',
			'hr',
			'i',
			'iframe',
			'img',
			'input',
			'ins',
			'kbd',
			'li',
			'main',
			'ol',
			'p',
			'pre',
			'span',
			'strike',
			'strong',
			'sub',
			'summary',
			'sup',
			'table',
			'tbody',
			'td',
			'th',
			'thead',
			'tr',
			'u',
			'ul',
		],
		filter: ({ tag, attrs }): boolean => {
			if (tag === 'iframe') {
				return /^https?:\/\/(www\.)?(youtube|youtube-nocookie)\.com\/embed\/[a-zA-Z0-9_]{11}(\?)?(&modestbranding=1)?(&autoplay=0)?(&loop=1)?(&playlist=[a-zA-Z0-9_]{11})?(&rel=0)?$/.test(
					attrs.src || ''
				)
			} else if (['h1', 'h2', 'h3', 'h4', 'h5', 'h6'].includes(tag)) {
				return attrs.id !== 'svelte'
			} else if (tag === 'input') {
				return attrs.type === 'checkbox' && attrs.disabled === ''
			} else if (tag === 'code' || tag === 'span') {
				return !attrs.class || attrs.class.replace(' ', '').startsWith('hljs')
			} else {
				return true
			}
		},
		transformText: null,
	})
}

export function markdownInline(markdown: string): string {
	return insane(
		marked.parseInline(markdown),
		{
			allowedAttributes: {
				a: ['href', 'target', 'title', 'rel'],
			},
			allowedClasses: {},
			allowedSchemes: ['http', 'https', 'mailto'],
			allowedTags: ['a', 'b', 'br', 'code', 'em', 'i', 'strike', 'strong', 'sub', 'sup', 'u'],
			transformText: null,
		},
		true
	)
}

export function markdown(markdown: string): string {
	return sanitize(marked.parse(markdown))
}
