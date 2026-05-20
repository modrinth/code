import { compareImportSources } from '@modrinth/tooling-config/script-utils/import-sort'
import fs from 'fs'
import path from 'path'

function toPascalCase(str: string): string {
	return str
		.split(/[-_.]/)
		.filter((part) => part.length > 0)
		.map((word) => {
			if (/^\d/.test(word)) {
				return word.charAt(0).toUpperCase() + word.slice(1)
			}
			return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase()
		})
		.join('')
}

function generateIconExports(): {
	imports: string
	exports: string
	categoryMap: string
	loaderMap: string
} {
	const packageRoot = path.resolve(__dirname, '..')
	const iconsDir = path.join(packageRoot, 'icons')

	if (!fs.existsSync(iconsDir)) {
		throw new Error(`Icons directory not found: ${iconsDir}`)
	}

	const icons: Array<{ importPath: string; pascalName: string; privateName: string }> = []
	const categoryMapEntries: Array<{ key: string; value: string }> = []
	const loaderMapEntries: Array<{ key: string; value: string }> = []

	// Process top-level icons
	const files = fs.readdirSync(iconsDir).filter((file) => {
		const filePath = path.join(iconsDir, file)
		return fs.statSync(filePath).isFile() && file.endsWith('.svg')
	})

	files.forEach((file) => {
		const baseName = path.basename(file, '.svg')
		let pascalName = toPascalCase(baseName)

		if (pascalName === '') {
			pascalName = 'Unknown'
		}

		if (!pascalName.endsWith('Icon')) {
			pascalName += 'Icon'
		}

		icons.push({
			importPath: `./icons/${file}?component`,
			pascalName,
			privateName: `_${pascalName}`,
		})
	})

	// Process tag icons from icons/tags/categories/
	const categoriesDir = path.join(iconsDir, 'tags', 'categories')
	if (fs.existsSync(categoriesDir)) {
		const categoryFiles = fs.readdirSync(categoriesDir).filter((file) => file.endsWith('.svg'))
		categoryFiles.forEach((file) => {
			const baseName = path.basename(file, '.svg')
			let pascalName = toPascalCase(baseName)

			if (pascalName === '') {
				pascalName = 'Unknown'
			}

			// Prefix with TagCategory
			pascalName = `TagCategory${pascalName}`
			if (!pascalName.endsWith('Icon')) {
				pascalName += 'Icon'
			}

			icons.push({
				importPath: `./icons/tags/categories/${file}?component`,
				pascalName,
				privateName: `_${pascalName}`,
			})

			// Add to category map (key is the original filename without extension, lowercase)
			categoryMapEntries.push({
				key: baseName.toLowerCase(),
				value: pascalName,
			})
		})
	}

	// Process tag icons from icons/tags/loaders/
	const loadersDir = path.join(iconsDir, 'tags', 'loaders')
	if (fs.existsSync(loadersDir)) {
		const loaderFiles = fs.readdirSync(loadersDir).filter((file) => file.endsWith('.svg'))
		loaderFiles.forEach((file) => {
			const baseName = path.basename(file, '.svg')
			let pascalName = toPascalCase(baseName)

			if (pascalName === '') {
				pascalName = 'Unknown'
			}

			// Prefix with TagLoader
			pascalName = `TagLoader${pascalName}`
			if (!pascalName.endsWith('Icon')) {
				pascalName += 'Icon'
			}

			icons.push({
				importPath: `./icons/tags/loaders/${file}?component`,
				pascalName,
				privateName: `_${pascalName}`,
			})

			// Add to loader map (key is the original filename without extension, lowercase)
			loaderMapEntries.push({
				key: baseName.toLowerCase(),
				value: pascalName,
			})
		})
	}

	// Sort by import path using simple-import-sort's algorithm
	icons.sort((a, b) => compareImportSources(a.importPath, b.importPath))

	// Sort map entries by key for consistent output
	categoryMapEntries.sort((a, b) => a.key.localeCompare(b.key))
	loaderMapEntries.sort((a, b) => a.key.localeCompare(b.key))

	let imports = ''
	let exports = ''

	icons.forEach(({ importPath, pascalName, privateName }) => {
		imports += `import ${privateName} from '${importPath}'\n`
		exports += `export const ${pascalName} = ${privateName}\n`
	})

	// Generate category map
	let categoryMap = 'export const categoryIconMap: Record<string, IconComponent> = {\n'
	categoryMapEntries.forEach(({ key, value }) => {
		categoryMap += `\t'${key}': ${value},\n`
	})
	categoryMap += '}\n'

	// Generate loader map
	let loaderMap = 'export const loaderIconMap: Record<string, IconComponent> = {\n'
	loaderMapEntries.forEach(({ key, value }) => {
		loaderMap += `\t'${key}': ${value},\n`
	})
	loaderMap += '}\n'

	return { imports, exports, categoryMap, loaderMap }
}

function runTests(): void {
	console.log('🧪 Running conversion tests...\n')

	const testCases: Array<{ input: string; expected: string }> = [
		{ input: 'align-left', expected: 'AlignLeftIcon' },
		{ input: 'arrow-big-up-dash', expected: 'ArrowBigUpDashIcon' },
		{ input: 'check-check', expected: 'CheckCheckIcon' },
		{ input: 'chevron-left', expected: 'ChevronLeftIcon' },
		{ input: 'file-archive', expected: 'FileArchiveIcon' },
		{ input: 'heart-handshake', expected: 'HeartHandshakeIcon' },
		{ input: 'monitor-smartphone', expected: 'MonitorSmartphoneIcon' },
		{ input: 'x-circle', expected: 'XCircleIcon' },
		{ input: 'rotate-ccw', expected: 'RotateCcwIcon' },
		{ input: 'bell-ring', expected: 'BellRingIcon' },
		{ input: 'more-horizontal', expected: 'MoreHorizontalIcon' },
		{ input: 'list_bulleted', expected: 'ListBulletedIcon' },
		{ input: 'test.name', expected: 'TestNameIcon' },
		{ input: 'test-name_final.icon', expected: 'TestNameFinalIcon' },
	]

	let passed = 0
	let failed = 0

	testCases.forEach(({ input, expected }) => {
		const result = toPascalCase(input) + (toPascalCase(input).endsWith('Icon') ? '' : 'Icon')
		const success = result === expected

		if (success) {
			console.log(`✅ ${input} → ${result}`)
			passed++
		} else {
			console.log(`❌ ${input} → ${result} (expected: ${expected})`)
			failed++
		}
	})

	console.log(`\n📊 Test Results: ${passed} passed, ${failed} failed`)

	if (failed > 0) {
		process.exit(1)
	}
}

function generateFiles(): void {
	try {
		console.log('🔄 Generating icon exports...')

		const { imports, exports, categoryMap, loaderMap } = generateIconExports()
		const output = `// Auto-generated icon imports and exports
// Do not edit this file manually - run 'pnpm run fix' to regenerate

import type { FunctionalComponent, SVGAttributes } from 'vue'

export type IconComponent = FunctionalComponent<SVGAttributes>

${imports}
${exports}

${categoryMap}
${loaderMap}`

		const packageRoot = path.resolve(__dirname, '..')
		const outputPath = path.join(packageRoot, 'generated-icons.ts')
		fs.writeFileSync(outputPath, output)

		console.log(`✅ Generated icon exports to: ${outputPath}`)
		console.log(
			`📦 Generated ${imports.split('\n').filter((line) => line.trim()).length} icon imports/exports`,
		)
	} catch (error) {
		console.error('❌ Error generating icons:', error)
		process.exit(1)
	}
}

function main(): void {
	const args = process.argv.slice(2)

	if (args.includes('--test')) {
		runTests()
	} else if (args.includes('--validate')) {
		validateIconConsistency()
	} else {
		generateFiles()
	}
}

main()

function getExpectedIconExports(iconsDir: string): string[] {
	if (!fs.existsSync(iconsDir)) {
		return []
	}

	const exports: string[] = []

	// Process top-level icons
	const files = fs.readdirSync(iconsDir).filter((file) => {
		const filePath = path.join(iconsDir, file)
		return fs.statSync(filePath).isFile() && file.endsWith('.svg')
	})

	files.forEach((file) => {
		const baseName = path.basename(file, '.svg')
		let pascalName = toPascalCase(baseName)

		if (pascalName === '') {
			pascalName = 'Unknown'
		}

		if (!pascalName.endsWith('Icon')) {
			pascalName += 'Icon'
		}

		exports.push(pascalName)
	})

	// Process tag icons from icons/tags/categories/
	const categoriesDir = path.join(iconsDir, 'tags', 'categories')
	if (fs.existsSync(categoriesDir)) {
		const categoryFiles = fs.readdirSync(categoriesDir).filter((file) => file.endsWith('.svg'))
		categoryFiles.forEach((file) => {
			const baseName = path.basename(file, '.svg')
			let pascalName = toPascalCase(baseName)

			if (pascalName === '') {
				pascalName = 'Unknown'
			}

			pascalName = `TagCategory${pascalName}`
			if (!pascalName.endsWith('Icon')) {
				pascalName += 'Icon'
			}

			exports.push(pascalName)
		})
	}

	// Process tag icons from icons/tags/loaders/
	const loadersDir = path.join(iconsDir, 'tags', 'loaders')
	if (fs.existsSync(loadersDir)) {
		const loaderFiles = fs.readdirSync(loadersDir).filter((file) => file.endsWith('.svg'))
		loaderFiles.forEach((file) => {
			const baseName = path.basename(file, '.svg')
			let pascalName = toPascalCase(baseName)

			if (pascalName === '') {
				pascalName = 'Unknown'
			}

			pascalName = `TagLoader${pascalName}`
			if (!pascalName.endsWith('Icon')) {
				pascalName += 'Icon'
			}

			exports.push(pascalName)
		})
	}

	return exports.sort()
}

function getActualIconExports(indexFile: string): string[] {
	if (!fs.existsSync(indexFile)) {
		return []
	}

	const content = fs.readFileSync(indexFile, 'utf8')
	const exportMatches = content.match(/export const (\w+Icon) = _\w+Icon/g) || []

	return exportMatches
		.map((match) => {
			const result = match.match(/export const (\w+Icon)/)
			return result ? result[1] : ''
		})
		.filter((name) => name.endsWith('Icon'))
		.sort()
}

function validateIconConsistency(): void {
	try {
		console.log('🔍 Validating icon consistency...')

		const packageRoot = path.resolve(__dirname, '..')
		const iconsDir = path.join(packageRoot, 'icons')
		const declarationFile = path.join(packageRoot, 'generated-icons.ts')

		const expectedExports = getExpectedIconExports(iconsDir)
		const actualExports = getActualIconExports(declarationFile)

		const missingExports = expectedExports.filter((name) => !actualExports.includes(name))
		const extraExports = actualExports.filter((name) => !expectedExports.includes(name))

		if (missingExports.length > 0) {
			console.error(`❌ Missing icon exports: ${missingExports.join(', ')}`)
			console.error("Run 'pnpm run fix' to generate them.")
			process.exit(1)
		}

		if (extraExports.length > 0) {
			console.error(
				`❌ Extra icon exports (no corresponding SVG files): ${extraExports.join(', ')}`,
			)
			console.error("Run 'pnpm run fix' to clean them up.")
			process.exit(1)
		}

		console.log('✅ Icon exports are consistent with SVG files')
	} catch (error) {
		console.error('❌ Error validating icons:', error)
		process.exit(1)
	}
}
