export function classCombine(names: string[]) {
	return names.filter((name) => name && !name.includes('undefined')).join(' ')
}
