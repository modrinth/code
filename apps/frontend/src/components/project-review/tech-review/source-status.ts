/** Distinguishes usable source from empty output and decompiler diagnostic placeholders. */
export function isUsableDecompiledSource(source: string | null | undefined): source is string {
	if (!source?.trim()) return false
	const content = source.replace(/\/\*[\s\S]*?\*\/|\/\/[^\r\n]*/g, '').trim()
	if (!content) return false
	return !/^(?:(?:[\w.$/-]+)\s+)?(?:took too long to decompile|failed to decompile|unable to decompile|could not decompile|decompilation (?:failed|error|timed out)|decompiler (?:failed|error|timed out)|manual analysis is required|(?:java\.[\w.]+(?:Exception|Error))\s*:)/i.test(
		content,
	)
}

// TODO - to make this check better, eventually labrinth will get a boolean if it decompiles correctly and we'll use that instead of this string check
