export function markdownToPlainText(markdown: string): string {
	if (!markdown) return ''

	const withoutCode = markdown.replace(/```[\s\S]*?```/g, ' ').replace(/`[^`]*`/g, ' ')
	const withoutImagesAndLinks = withoutCode
		.replace(/!\[[^\]]*]\([^)]+\)/g, ' ')
		.replace(/\[([^\]]*)]\([^)]+\)/g, '$1')
	const withoutHtml = withoutImagesAndLinks.replace(/<[^>]+>/g, ' ')
	const withoutMarkdownSyntax = withoutHtml
		.replace(/^>{1}\s?.*$/gm, ' ')
		.replace(/^#{1,6}\s+/gm, ' ')
		.replace(/[*_~`>-]/g, ' ')
		.replace(/\|/g, ' ')

	return withoutMarkdownSyntax.replace(/\s+/g, ' ').trim()
}

export function countMarkdownText(markdown: string): number {
	return markdownToPlainText(markdown).length
}

export function markdownHasImage(markdown: string): boolean {
	if (!markdown) return false
	return /!\[[^\]]*]\([^)]+\)/.test(markdown) || /<img[\s>]/i.test(markdown)
}
