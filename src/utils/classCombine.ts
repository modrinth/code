export function classCombine(names) {
	return names.filter((name) => name && !name.includes('undefined')).join(' ')
}
