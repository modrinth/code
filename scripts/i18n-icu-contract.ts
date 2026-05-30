import { Client as CrowdinClient, type Credentials } from '@crowdin/crowdin-api-client'
import { parse, TYPE } from '@formatjs/icu-messageformat-parser'
import { execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { readFile, readdir, writeFile } from 'node:fs/promises'
import { basename, dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { parse as parseYaml } from 'yaml'

type MessageEntry = string | { message?: string; defaultMessage?: string }
type MessageFile = Record<string, MessageEntry>
type CrowdinFileEntry = { source: string; dest?: string; translation: string }
type Contract = { args: string[]; tags: string[]; branches: string[] }
type Issue = { file: string; key: string; reason: string }
type CrowdinListResponse<T> = {
	data: Array<{ data: T }>
	pagination: { offset: number; limit: number }
}

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const DEFAULT_LOCALE = 'en-US'

function stripLeadingSlash(path: string) {
	return path.replace(/^[/\\]+/, '')
}

function normalizeCrowdinPath(path: string) {
	const normalized = path.replaceAll('\\', '/').replace(/^\/?/, '/')
	return normalized.replaceAll('//', '/')
}

function textOf(entry: MessageEntry | undefined): string | undefined {
	if (typeof entry === 'string') return entry
	return entry?.message ?? entry?.defaultMessage
}

function stable(items: Set<string>) {
	return [...items].sort()
}

export function contractFromMessage(message: string, label: string): Contract {
	const args = new Set<string>()
	const tags = new Set<string>()
	const branches = new Set<string>()

	function visit(elements: ReturnType<typeof parse>) {
		for (const element of elements) {
			switch (element.type) {
				case TYPE.argument:
					args.add(`${element.value}:argument`)
					break
				case TYPE.number:
					args.add(`${element.value}:number`)
					break
				case TYPE.date:
					args.add(`${element.value}:date`)
					break
				case TYPE.time:
					args.add(`${element.value}:time`)
					break
				case TYPE.select: {
					args.add(`${element.value}:select`)
					for (const [selector, option] of Object.entries(element.options)) {
						branches.add(`${element.value}:select:${selector}`)
						visit(option.value)
					}
					break
				}
				case TYPE.plural: {
					args.add(`${element.value}:plural:${element.pluralType}`)
					for (const [selector, option] of Object.entries(element.options)) {
						branches.add(`${element.value}:plural:${selector}`)
						visit(option.value)
					}
					break
				}
				case TYPE.tag:
					tags.add(element.value)
					visit(element.children)
					break
			}
		}
	}

	try {
		visit(parse(message, { ignoreTag: false }))
	} catch (error) {
		try {
			visit(parse(message, { ignoreTag: true }))
		} catch {
			throw new Error(`${label}: invalid ICU: ${(error as Error).message}`)
		}
	}

	return { args: stable(args), tags: stable(tags), branches: stable(branches) }
}

export function contractsEqual(a: Contract, b: Contract) {
	return JSON.stringify(a) === JSON.stringify(b)
}

export function sourceContractChanged(
	previousText: string,
	currentText: string,
	previousLabel: string,
	currentLabel: string,
) {
	const after = contractFromMessage(currentText, currentLabel)

	try {
		const before = contractFromMessage(previousText, previousLabel)
		return !contractsEqual(before, after)
	} catch {
		return true
	}
}

async function readJson(file: string): Promise<MessageFile> {
	return JSON.parse(await readFile(file, 'utf8')) as MessageFile
}

async function writeJson(file: string, value: MessageFile) {
	await writeFile(file, `${JSON.stringify(value, null, 2)}\n`)
}

async function loadCrowdinEntries(scope?: string) {
	const raw = await readFile(resolve(ROOT, 'crowdin.yml'), 'utf8')
	const config = parseYaml(raw) as { files: CrowdinFileEntry[] }
	return config.files.filter((entry) => {
		if (!scope) return true
		return stripLeadingSlash(entry.source).startsWith(`${scope.replace(/\/$/, '')}/`)
	})
}

async function sourceFilesFor(entry: CrowdinFileEntry) {
	const source = stripLeadingSlash(entry.source)
	if (!source.endsWith('*.json')) return [resolve(ROOT, source)]

	const sourceDir = resolve(ROOT, source.slice(0, -'*.json'.length))
	const files = await readdir(sourceDir)
	return files.filter((file) => file.endsWith('.json')).map((file) => join(sourceDir, file))
}

async function translationFilesFor(entry: CrowdinFileEntry, sourceFile: string) {
	const template = stripLeadingSlash(entry.translation)
	const localeIndex = template.indexOf('%locale%')
	if (localeIndex === -1) throw new Error(`Translation path lacks %locale%: ${entry.translation}`)

	const beforeLocale = template.slice(0, localeIndex)
	const afterLocale = template
		.slice(localeIndex + '%locale%'.length)
		.replace(/^[/\\]+/, '')
		.replaceAll('%original_file_name%', basename(sourceFile))

	const localeRoot = resolve(ROOT, beforeLocale)
	const dirs = await readdir(localeRoot, { withFileTypes: true })

	return dirs
		.filter((dir) => dir.isDirectory() && dir.name !== DEFAULT_LOCALE)
		.map((dir) => join(localeRoot, dir.name, afterLocale))
}

function sourceContracts(sourceFile: string, sourceMessages: MessageFile) {
	const contracts = new Map<string, Contract>()
	for (const [key, value] of Object.entries(sourceMessages)) {
		const text = textOf(value)
		if (text === undefined) throw new Error(`${sourceFile}:${key}: missing source message`)
		contracts.set(key, contractFromMessage(text, `${sourceFile}:${key}`))
	}
	return contracts
}

export async function pruneLocalTranslations(options: { check: boolean; scope?: string }) {
	const issues: Issue[] = []
	const entries = await loadCrowdinEntries(options.scope)

	for (const entry of entries) {
		for (const sourceFile of await sourceFilesFor(entry)) {
			const source = await readJson(sourceFile)
			const contracts = sourceContracts(sourceFile, source)

			for (const translationFile of await translationFilesFor(entry, sourceFile)) {
				if (!existsSync(translationFile)) continue

				const translations = await readJson(translationFile)
				let changed = false

				for (const [key, value] of Object.entries(translations)) {
					const sourceContract = contracts.get(key)
					const translationText = textOf(value)

					if (!sourceContract) {
						delete translations[key]
						changed = true
						issues.push({ file: translationFile, key, reason: 'source key no longer exists' })
						continue
					}

					if (translationText === undefined) {
						delete translations[key]
						changed = true
						issues.push({ file: translationFile, key, reason: 'translation has no message text' })
						continue
					}

					try {
						const translationContract = contractFromMessage(translationText, `${translationFile}:${key}`)
						if (!contractsEqual(sourceContract, translationContract)) {
							delete translations[key]
							changed = true
							issues.push({ file: translationFile, key, reason: 'ICU contract differs from en-US' })
						}
					} catch {
						delete translations[key]
						changed = true
						issues.push({ file: translationFile, key, reason: 'translation ICU is invalid' })
					}
				}

				if (changed && !options.check) await writeJson(translationFile, translations)
			}
		}
	}

	for (const issue of issues) {
		console.log(`${relative(ROOT, issue.file)}: ${issue.key} - ${issue.reason}`)
	}

	if (options.check && issues.length > 0) {
		throw new Error(`${issues.length} stale i18n translation(s) need pruning`)
	}
}

function gitFile(ref: string, file: string) {
	const rel = relative(ROOT, file).replaceAll('\\', '/')
	try {
		return execFileSync('git', ['show', `${ref}:${rel}`], {
			cwd: ROOT,
			encoding: 'utf8',
			stdio: ['ignore', 'pipe', 'ignore'],
		})
	} catch {
		return null
	}
}

function crowdinDestPath(entry: CrowdinFileEntry, sourceFile: string) {
	const dest = entry.dest ?? entry.source
	return normalizeCrowdinPath(dest.replaceAll('%original_file_name%', basename(sourceFile)))
}

async function changedSourceIds(baseRef: string, scope?: string) {
	const changed = new Map<string, Set<string>>()

	for (const entry of await loadCrowdinEntries(scope)) {
		for (const sourceFile of await sourceFilesFor(entry)) {
			const previousRaw = gitFile(baseRef, sourceFile)
			if (!previousRaw) continue

			const current = await readJson(sourceFile)
			const previous = JSON.parse(previousRaw) as MessageFile
			const destPath = crowdinDestPath(entry, sourceFile)

			for (const [key, currentEntry] of Object.entries(current)) {
				const previousText = textOf(previous[key])
				const currentText = textOf(currentEntry)
				if (previousText === undefined || currentText === undefined) continue

				if (
					sourceContractChanged(
						previousText,
						currentText,
						`${baseRef}:${sourceFile}:${key}`,
						`${sourceFile}:${key}`,
					)
				) {
					const ids = changed.get(destPath) ?? new Set<string>()
					ids.add(key)
					changed.set(destPath, ids)
				}
			}
		}
	}

	return changed
}

async function listAll<T>(
	load: (limit: number, offset: number) => Promise<CrowdinListResponse<T>>,
) {
	const all: T[] = []
	let offset = 0
	const limit = 500

	for (;;) {
		const response = await load(limit, offset)
		const page = response.data.map((item) => item.data)
		all.push(...page)

		const pageLimit = response.pagination.limit || limit
		if (page.length < pageLimit) return all
		offset += pageLimit
	}
}

export async function clearCrowdinChangedTranslations(options: {
	baseRef: string
	crowdinBranch: string
	scope?: string
}) {
	const projectId = Number(process.env.CROWDIN_PROJECT_ID)
	const token = process.env.CROWDIN_PERSONAL_TOKEN
	if (!projectId || !token) throw new Error('CROWDIN_PROJECT_ID and CROWDIN_PERSONAL_TOKEN are required')

	const changed = await changedSourceIds(options.baseRef, options.scope)
	if (changed.size === 0) {
		console.log('No ICU contract changes found.')
		return
	}

	const credentials: Credentials = { token }
	const client = new CrowdinClient(credentials)
	const branches = await listAll((limit, offset) =>
		client.sourceFilesApi.listProjectBranches(projectId, {
			name: options.crowdinBranch,
			limit,
			offset,
		}) as Promise<CrowdinListResponse<{ id: number; name: string }>>,
	)
	const branch = branches.find((item) => item.name === options.crowdinBranch)
	if (!branch) throw new Error(`Crowdin branch not found: ${options.crowdinBranch}`)

	const files = await listAll((limit, offset) =>
		client.sourceFilesApi.listProjectFiles(projectId, {
			branchId: branch.id,
			recursion: 1,
			limit,
			offset,
		}) as Promise<CrowdinListResponse<{ id: number; path: string }>>,
	)
	const fileByPath = new Map(files.map((file) => [normalizeCrowdinPath(file.path), file]))

	for (const [destPath, keys] of changed) {
		const file = fileByPath.get(destPath)
		if (!file) throw new Error(`Crowdin file not found: ${destPath}`)

		const strings = await listAll((limit, offset) =>
			client.sourceStringsApi.listProjectStrings(projectId, {
				branchId: branch.id,
				fileId: file.id,
				limit,
				offset,
			}) as Promise<CrowdinListResponse<{ id: number; identifier: string }>>,
		)
		const stringByIdentifier = new Map(strings.map((sourceString) => [sourceString.identifier, sourceString]))

		for (const key of keys) {
			const sourceString = stringByIdentifier.get(key)
			if (!sourceString) throw new Error(`Crowdin string not found: ${destPath}:${key}`)
			await client.stringTranslationsApi.deleteAllTranslations(projectId, sourceString.id)
			console.log(`Cleared translations for ${destPath}:${key}`)
		}
	}
}

function readOptions(args: string[]) {
	const options: Record<string, string | boolean> = {}
	for (let i = 0; i < args.length; i++) {
		const arg = args[i]
		if (!arg.startsWith('--')) continue
		const key = arg.slice(2)
		const next = args[i + 1]
		if (!next || next.startsWith('--')) {
			options[key] = true
		} else {
			options[key] = next
			i++
		}
	}
	return options
}

async function main() {
	const [command, ...rest] = process.argv.slice(2)
	const options = readOptions(rest)

	if (command === 'prune-local') {
		await pruneLocalTranslations({
			check: options.check === true,
			scope: typeof options.scope === 'string' ? options.scope : undefined,
		})
		return
	}

	if (command === 'clear-crowdin-changed') {
		await clearCrowdinChangedTranslations({
			baseRef: typeof options['base-ref'] === 'string' ? options['base-ref'] : 'HEAD^',
			crowdinBranch:
				typeof options['crowdin-branch'] === 'string'
					? options['crowdin-branch']
					: (() => {
							throw new Error('--crowdin-branch is required')
						})(),
			scope: typeof options.scope === 'string' ? options.scope : undefined,
		})
		return
	}

	throw new Error('Usage: pnpm scripts i18n-icu-contract prune-local|clear-crowdin-changed')
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
	main().catch((error) => {
		console.error(error)
		process.exit(1)
	})
}
