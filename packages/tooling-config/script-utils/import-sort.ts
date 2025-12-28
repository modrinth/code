/**
 * Import sorting utility that matches eslint-plugin-simple-import-sort's algorithm.
 * Use this to pre-sort generated imports so ESLint doesn't need to reformat them - which can cause it to have different diffs
 */

const collator = new Intl.Collator('en', {
	sensitivity: 'base',
	numeric: true,
})

function compare(a: string, b: string): number {
	return collator.compare(a, b) || (a < b ? -1 : a > b ? 1 : 0)
}

/**
 * Transforms an import source path the same way simple-import-sort does internally.
 * This swaps certain characters to achieve the desired sort order:
 * - `.` and `/` sort before other punctuation
 * - `..` sorts like `../,` to come after `../../` but before `../a`
 */
function transformSource(source: string): string {
	return source
		.replace(/^[./]*\.$/, '$&/')
		.replace(/^[./]*\/$/, '$&,')
		.replace(/[./_-]/g, (char) => {
			switch (char) {
				case '.':
					return '_'
				case '/':
					return '-'
				case '_':
					return '.'
				case '-':
					return '/'
				default:
					return char
			}
		})
}

/**
 * Compares two import source paths using the same algorithm as simple-import-sort.
 * Use this as a comparator function for Array.sort().
 *
 * @example
 * const imports = ['./foo', './bar', './baz'];
 * imports.sort(compareImportSources);
 */
export function compareImportSources(a: string, b: string): number {
	return compare(transformSource(a), transformSource(b))
}

/**
 * Sorts an array of import source paths using the same algorithm as simple-import-sort.
 *
 * @example
 * const sorted = sortImportSources(['./z', './a', './m']);
 * // Returns: ['./a', './m', './z']
 */
export function sortImportSources<T extends string>(sources: T[]): T[] {
	return sources.slice().sort(compareImportSources)
}

/**
 * Sorts an array of items by their import source path.
 *
 * @example
 * const items = [{ path: './z' }, { path: './a' }];
 * const sorted = sortByImportSource(items, item => item.path);
 */
export function sortByImportSource<T>(items: T[], getSource: (item: T) => string): T[] {
	return items.slice().sort((a, b) => compareImportSources(getSource(a), getSource(b)))
}
