export function getStringFormatParser(language: string) {
	switch (language) {
		case 'javascript':
			return 'babel'
		case 'json':
			return 'json'
		case 'xml':
			return 'html'
	}
}

export async function formatStringContents(
	text: string,
	parser: NonNullable<ReturnType<typeof getStringFormatParser>>,
) {
	const prettier = await import('prettier/standalone')
	const plugins = await Promise.all([
		import('prettier/plugins/babel'),
		import('prettier/plugins/estree'),
		...(parser === 'html'
			? [import('prettier/plugins/html'), import('prettier/plugins/postcss')]
			: []),
	])
	return prettier.format(text, { parser, plugins, useTabs: true })
}
