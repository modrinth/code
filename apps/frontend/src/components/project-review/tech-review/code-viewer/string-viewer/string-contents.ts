export function decodeStringContents(literal: string): string | undefined {
	if (literal.startsWith('"')) {
		try {
			const value: unknown = JSON.parse(literal)
			if (typeof value === 'string') return value
		} catch {
			return decodeJavaString(literal)
		}
	}
	return decodeJavaString(literal)
}

function decodeJavaString(literal: string): string | undefined {
	const quote = literal[0]
	if ((quote !== '"' && quote !== "'") || !literal.endsWith(quote) || literal.length < 2) return
	const body = literal.slice(1, -1)
	const escapes: Record<string, string> = {
		b: '\b',
		t: '\t',
		n: '\n',
		f: '\f',
		r: '\r',
		s: ' ',
		'"': '"',
		"'": "'",
		'\\': '\\',
	}
	let result = ''
	for (let index = 0; index < body.length; index++) {
		const character = body[index]
		if (character === quote || character === '\n' || character === '\r') return
		if (character !== '\\') {
			result += character
			continue
		}
		const escape = body[++index]
		if (escape === undefined) return
		if (escape === 'u') {
			while (body[index + 1] === 'u') index++
			const hex = body.slice(index + 1, index + 5)
			if (!/^[\da-f]{4}$/i.test(hex)) return
			result += String.fromCharCode(parseInt(hex, 16))
			index += 4
		} else if (/[0-7]/.test(escape)) {
			const octal = body.slice(index).match(escape <= '3' ? /^[0-7]{1,3}/ : /^[0-7]{1,2}/)![0]
			result += String.fromCharCode(parseInt(octal, 8))
			index += octal.length - 1
		} else if (Object.hasOwn(escapes, escape)) {
			result += escapes[escape]
		} else return
	}
	return result
}
