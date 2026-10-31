export function formatCodeBlock(code: string, language = '') {
	const lines = code.replace(/\r\n?/g, '\n').split('\n')
	while (lines.length && !lines[0].trim()) lines.shift()
	while (lines.length && !lines[lines.length - 1].trim()) lines.pop()
	const nonEmptyLines = lines.filter((line) => line.trim())
	let indentation = nonEmptyLines[0]?.match(/^[\t ]*/)?.[0] ?? ''
	for (const line of nonEmptyLines) {
		while (indentation && !line.startsWith(indentation)) {
			indentation = indentation.slice(0, -1)
		}
	}
	code = lines.map((line) => (line.trim() ? line.slice(indentation.length) : '')).join('\n')
	const fenceLength = Array.from(code.matchAll(/`+/g)).reduce(
		(length, [match]) => Math.max(length, match.length + 1),
		3,
	)
	const fence = '`'.repeat(fenceLength)
	return `${fence}${language}\n${code}\n${fence}\n`
}

export function codeClipboardHtml(document: Document, code: string, language: string) {
	const pre = document.createElement('pre')
	pre.dataset.modrinthCodeLanguage = language
	const content = document.createElement('code')
	content.textContent = code
	pre.append(content)
	return pre.outerHTML
}

export function readCodeClipboard(document: Document, clipboard: DataTransfer) {
	const html = clipboard.getData('text/html')
	const Parser = document.defaultView?.DOMParser
	if (!html || !Parser) return
	const parsed = new Parser().parseFromString(html, 'text/html')
	const pre = parsed.querySelector('pre[data-modrinth-code-language]')
	if (!pre) return
	const language = pre.getAttribute('data-modrinth-code-language') ?? ''
	return formatCodeBlock(
		clipboard.getData('text/plain'),
		/^[\w+-]*$/.test(language) ? language : '',
	)
}
