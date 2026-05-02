#!/usr/bin/env node
import { readFile } from 'node:fs/promises'
import { basename, extname, resolve } from 'node:path'

const DEFAULT_PROD_API = 'https://api.modrinth.com/v3'
const DEFAULT_LOCAL_API = 'http://127.0.0.1:8000/v3'
const DEFAULT_TOKEN_FIXTURE =
	new URL('../apps/labrinth/fixtures/labrinth-seed-data-202508052143.sql', import.meta.url)
		.pathname
const USER_AGENT = 'modrinth-local-labrinth-cloner/1.0'
const LOCAL_ICON_LIMIT_BYTES = 256 * 1024
const DEFAULT_MAX_FILE_BYTES = 256 * 1024 * 1024

const VERSION_CREATE_CORE_FIELDS = new Set([
	'id',
	'project_id',
	'author_id',
	'featured',
	'name',
	'version_title',
	'version_number',
	'project_types',
	'games',
	'changelog',
	'version_body',
	'date_published',
	'downloads',
	'version_type',
	'release_channel',
	'status',
	'requested_status',
	'files',
	'dependencies',
	'loaders',
	'ordering',
])

class HttpError extends Error {
	constructor(method, url, status, body) {
		const detail =
			body && typeof body === 'object'
				? body.description || body.error || JSON.stringify(body)
				: body
		super(`${method} ${url} failed with ${status}${detail ? `: ${detail}` : ''}`)
		this.status = status
		this.body = body
	}
}

class SkipVersionError extends Error {}

function usage() {
	console.log(`Usage:
  node scripts/clone-labrinth-projects.mjs search <query> [options]
  node scripts/clone-labrinth-projects.mjs top [options]
  node scripts/clone-labrinth-projects.mjs project <id-or-slug...> [options]

Examples:
  node scripts/clone-labrinth-projects.mjs search sodium --limit 3 --versions 2
  node scripts/clone-labrinth-projects.mjs top --limit 20 --versions 1
  node scripts/clone-labrinth-projects.mjs project sodium lithium --versions all

Options:
  --prod-api <url>       Prod API base. Default: ${DEFAULT_PROD_API}
  --local-api <url>      Local API base. Default: ${DEFAULT_LOCAL_API}
  --token <token>        Local token. Default: LABRINTH_LOCAL_TOKEN or fixture mra_admin
  --token-fixture <path> SQL fixture to read mra_admin from
  --limit <n>            Search/top project count. Default: search=5, top=10
  --offset <n>           Search offset. Default: 0
  --index <name>         Search index. Default: relevance for search, downloads for top
  --facets <json>        Search facets JSON. Default: []
  --versions <n|all>     Versions to clone per project. Default: 1
  --version-status <s>   Local version status. Default: listed
  --slug-prefix <text>   Prefix applied to local slugs
  --slug-suffix <text>   Suffix applied to local slugs
  --max-file-mib <n>     Skip individual files larger than this. Default: 256
  --delay-ms <n>         Delay between API mutations. Default: 250
  --include-dependencies Preserve file-name-only dependencies. Default: false
  --no-icon              Do not copy project icons
  --dry-run              Fetch metadata and print actions without creating/uploading
  --help                 Show this help
`)
}

function parseArgs(argv) {
	const options = {
		prodApi: process.env.PROD_LABRINTH_API || DEFAULT_PROD_API,
		localApi: process.env.LOCAL_LABRINTH_API || DEFAULT_LOCAL_API,
		token: process.env.LABRINTH_LOCAL_TOKEN || null,
		tokenFixture: DEFAULT_TOKEN_FIXTURE,
		limit: null,
		offset: 0,
		index: null,
		facets: '[]',
		versionLimit: 1,
		versionStatus: 'listed',
		slugPrefix: '',
		slugSuffix: '',
		maxFileBytes: DEFAULT_MAX_FILE_BYTES,
		delayMs: 250,
		includeDependencies: false,
		includeIcon: true,
		dryRun: false,
	}
	const positionals = []

	for (let i = 0; i < argv.length; i++) {
		const arg = argv[i]
		if (!arg.startsWith('--')) {
			positionals.push(arg)
			continue
		}

		const [flag, inlineValue] = arg.split('=', 2)
		const readValue = () => {
			if (inlineValue !== undefined) return inlineValue
			i += 1
			if (i >= argv.length) throw new Error(`Missing value for ${flag}`)
			return argv[i]
		}

		switch (flag) {
			case '--prod-api':
				options.prodApi = readValue()
				break
			case '--local-api':
				options.localApi = readValue()
				break
			case '--token':
				options.token = readValue()
				break
			case '--token-fixture':
				options.tokenFixture = resolve(process.cwd(), readValue())
				break
			case '--limit':
				options.limit = parsePositiveInteger(readValue(), flag)
				break
			case '--offset':
				options.offset = parseNonNegativeInteger(readValue(), flag)
				break
			case '--index':
				options.index = readValue()
				break
			case '--facets':
				options.facets = readValue()
				JSON.parse(options.facets)
				break
			case '--versions': {
				const value = readValue()
				options.versionLimit =
					value === 'all' ? Number.POSITIVE_INFINITY : parsePositiveInteger(value, flag)
				break
			}
			case '--version-status':
				options.versionStatus = readValue()
				break
			case '--slug-prefix':
				options.slugPrefix = readValue()
				break
			case '--slug-suffix':
				options.slugSuffix = readValue()
				break
			case '--max-file-mib':
				options.maxFileBytes = parsePositiveInteger(readValue(), flag) * 1024 * 1024
				break
			case '--delay-ms':
				options.delayMs = parseNonNegativeInteger(readValue(), flag)
				break
			case '--include-dependencies':
				options.includeDependencies = true
				break
			case '--no-icon':
				options.includeIcon = false
				break
			case '--dry-run':
				options.dryRun = true
				break
			case '--help':
			case '-h':
				options.help = true
				break
			default:
				throw new Error(`Unknown option: ${flag}`)
		}
	}

	const [mode, ...modeArgs] = positionals
	options.mode = mode
	options.modeArgs = modeArgs

	return options
}

function parsePositiveInteger(value, flag) {
	const parsed = Number.parseInt(value, 10)
	if (!Number.isInteger(parsed) || parsed <= 0) {
		throw new Error(`${flag} must be a positive integer`)
	}
	return parsed
}

function parseNonNegativeInteger(value, flag) {
	const parsed = Number.parseInt(value, 10)
	if (!Number.isInteger(parsed) || parsed < 0) {
		throw new Error(`${flag} must be a non-negative integer`)
	}
	return parsed
}

async function main() {
	const options = parseArgs(process.argv.slice(2))
	if (options.help || !options.mode) {
		usage()
		return
	}

	if (!['search', 'top', 'project'].includes(options.mode)) {
		throw new Error(`Unknown mode: ${options.mode}`)
	}
	if (options.mode === 'search' && options.modeArgs.length === 0) {
		throw new Error('search mode requires a query')
	}
	if (options.mode === 'project' && options.modeArgs.length === 0) {
		throw new Error('project mode requires at least one project id or slug')
	}

	if (!options.token) {
		options.token = await readMraAdminToken(options.tokenFixture)
	}

	console.log(`Prod API:  ${options.prodApi}`)
	console.log(`Local API: ${options.localApi}`)
	console.log(`Local token: ${options.token}`)
	if (options.dryRun) console.log('Dry run enabled; no local mutations will be sent.')

	const localMetadata = await loadLocalMetadata(options)
	const projectRefs = await resolveProjectRefs(options)

	console.log(`Found ${projectRefs.length} project(s) to clone.`)
	for (const [index, ref] of projectRefs.entries()) {
		console.log(`\n[${index + 1}/${projectRefs.length}] ${ref}`)
		try {
			await cloneProject(ref, options, localMetadata)
		} catch (error) {
			console.error(`  Failed: ${error.message}`)
		}
		await sleep(options.delayMs)
	}
}

async function readMraAdminToken(fixturePath) {
	const sql = await readFile(fixturePath, 'utf8')
	const match = sql.match(/,\s*'(mra_admin)'\s*,/)
	if (!match) {
		throw new Error(`Could not find mra_admin session token in ${fixturePath}`)
	}
	return match[1]
}

async function loadLocalMetadata(options) {
	const [categories, loaders, gameVersions, environments, linkPlatforms] = await Promise.all([
		requestJson(options.localApi, '/tag/category'),
		requestJson(options.localApi, '/tag/loader'),
		requestJson(options.localApi, '/loader_field', {
			query: { loader_field: 'game_versions' },
		}),
		requestJson(options.localApi, '/loader_field', {
			query: { loader_field: 'environment' },
		}).catch(() => []),
		requestJson(options.localApi, '/link_platform').catch(() => []),
	])

	const gameVersionValues = gameVersions.map((value) => value.value).filter(Boolean)
	const fallbackGameVersion =
		gameVersions
			.filter((value) => value.type === 'release')
			.sort((a, b) => Date.parse(b.created) - Date.parse(a.created))[0]?.value ||
		gameVersionValues[0] ||
		'1.20.1'

	return {
		categories: new Set(categories.map((category) => category.name)),
		loaders: new Map(loaders.map((loader) => [loader.name, loader])),
		gameVersions: new Set(gameVersionValues),
		fallbackGameVersion,
		environments: new Set(environments.map((environment) => environment.value).filter(Boolean)),
		linkPlatforms: new Set(linkPlatforms.map((platform) => platform.name)),
	}
}

async function resolveProjectRefs(options) {
	if (options.mode === 'project') return [...new Set(options.modeArgs)]

	const limit = options.limit ?? (options.mode === 'top' ? 10 : 5)
	const query = {
		limit,
		offset: options.offset,
		index: options.index ?? (options.mode === 'top' ? 'downloads' : 'relevance'),
		facets: options.facets,
	}
	if (options.mode === 'search') {
		query.query = options.modeArgs.join(' ')
	}

	const search = await requestJson(options.prodApi, '/search', { query })
	const hits = Array.isArray(search.hits) ? search.hits : []
	return hits.map((hit) => hit.project_id || hit.id || hit.slug).filter(Boolean)
}

async function cloneProject(projectRef, options, localMetadata) {
	const prodProject = await requestJson(options.prodApi, `/project/${encodeURIComponent(projectRef)}`)
	const prodSlug = prodProject.slug || projectRef
	const localSlug = sanitizeSlug(`${options.slugPrefix}${prodSlug}${options.slugSuffix}`)
	const name = projectName(prodProject)

	console.log(`  Project: ${name} (${prodSlug} -> ${localSlug})`)

	const localProject =
		(await requestJson(options.localApi, `/project/${encodeURIComponent(localSlug)}`, {
			token: options.token,
			allow404: true,
		})) || (await createLocalProject(prodProject, localSlug, options, localMetadata))

	if (localProject.slug === localSlug) {
		console.log(`  Local project id: ${localProject.id}`)
	}

	const existingLocalVersions = await getLocalProjectVersions(localProject.id || localSlug, options)
	const existingVersionNumbers = new Set(
		existingLocalVersions.map((version) => version.version_number).filter(Boolean),
	)
	const prodVersions = await getProdProjectVersions(prodProject.id || projectRef, options)
	console.log(`  Versions selected: ${prodVersions.length}`)

	for (const version of prodVersions) {
		if (existingVersionNumbers.has(version.version_number)) {
			console.log(`  - ${version.version_number}: already exists locally, skipped`)
			continue
		}

		try {
			const created = await cloneVersion(prodProject, version, localProject, options, localMetadata)
			if (created) existingVersionNumbers.add(version.version_number)
		} catch (error) {
			if (error instanceof SkipVersionError) {
				console.log(`  - ${version.version_number}: skipped (${error.message})`)
			} else {
				console.error(`  - ${version.version_number}: failed (${error.message})`)
			}
		}
		await sleep(options.delayMs)
	}
}

async function createLocalProject(prodProject, localSlug, options, localMetadata) {
	const createData = {
		name: clampText(projectName(prodProject), 'Imported project', 3, 64),
		slug: localSlug,
		summary: clampText(projectSummary(prodProject), 'Imported from production Modrinth.', 3, 255),
		description: truncate(projectBody(prodProject), 65536),
		initial_versions: [],
		is_draft: true,
		categories: filterCategories(prodProject.categories, localMetadata, 3),
		additional_categories: filterCategories(prodProject.additional_categories, localMetadata, 256),
		license_id: licenseId(prodProject),
		license_url: prodProject.license?.url || prodProject.license_url || null,
		link_urls: linkUrls(prodProject, localMetadata),
		requested_status: 'approved',
	}
	createData.additional_categories = createData.additional_categories.filter(
		(category) => !createData.categories.includes(category),
	)

	if (options.dryRun) {
		console.log(`  Would create draft project ${localSlug}`)
		return { id: `dry-run-${localSlug}`, slug: localSlug }
	}

	const formData = new FormData()
	formData.append('data', JSON.stringify(createData))

	if (options.includeIcon && prodProject.icon_url) {
		const icon = await downloadIcon(prodProject.icon_url).catch((error) => {
			console.log(`  Icon skipped: ${error.message}`)
			return null
		})
		if (icon) {
			formData.append('icon', icon.blob, icon.filename)
		}
	}

	console.log('  Creating draft project via local API')
	return await requestJson(options.localApi, '/project', {
		method: 'POST',
		token: options.token,
		body: formData,
	})
}

async function getLocalProjectVersions(projectId, options) {
	if (options.dryRun) return []
	return (
		(await requestJson(options.localApi, `/project/${encodeURIComponent(projectId)}/version`, {
			token: options.token,
			query: { include_changelog: false },
			allow404: true,
		})) || []
	)
}

async function getProdProjectVersions(projectId, options) {
	if (options.versionLimit !== Number.POSITIVE_INFINITY) {
		return await requestJson(options.prodApi, `/project/${encodeURIComponent(projectId)}/version`, {
			query: {
				limit: options.versionLimit,
				offset: 0,
				include_changelog: true,
			},
		})
	}

	const versions = []
	const pageSize = 100
	for (let offset = 0; ; offset += pageSize) {
		const batch = await requestJson(options.prodApi, `/project/${encodeURIComponent(projectId)}/version`, {
			query: {
				limit: pageSize,
				offset,
				include_changelog: true,
			},
		})
		versions.push(...batch)
		if (batch.length < pageSize) break
	}
	return versions
}

async function cloneVersion(prodProject, prodVersion, localProject, options, localMetadata) {
	const prodFiles = Array.isArray(prodVersion.files) ? prodVersion.files : []
	if (prodFiles.length === 0) {
		throw new SkipVersionError('version has no files')
	}

	const primaryFile = prodFiles.find((file) => file.primary) || prodFiles[0]
	if (primaryFile.size && primaryFile.size > options.maxFileBytes) {
		throw new SkipVersionError(
			`primary file is ${formatBytes(primaryFile.size)}, above --max-file-mib`,
		)
	}

	if (options.dryRun) {
		console.log(
			`  - ${prodVersion.version_number}: would upload ${prodFiles.length} file(s) from prod`,
		)
		return { id: `dry-run-${prodVersion.id}` }
	}

	const uploadFiles = await downloadVersionFiles(prodFiles, options)
	const primaryUpload =
		uploadFiles.find((file) => file.source === primaryFile) || uploadFiles.find((file) => file.primary)
	if (!primaryUpload) {
		throw new SkipVersionError('primary file was not downloaded')
	}

	const versionFields = normalizeVersionFields(prodProject, prodVersion, primaryUpload, localMetadata)
	const fileParts = uploadFiles.map((file) => file.partName)
	const fileTypes = Object.fromEntries(
		uploadFiles.map((file) => [file.partName, file.source.file_type || null]),
	)

	const createData = {
		project_id: localProject.id,
		file_parts: fileParts,
		version_number: sanitizeVersionNumber(prodVersion.version_number),
		version_title: clampText(prodVersion.name || prodVersion.version_number, 'Imported version', 1, 64),
		version_body: prodVersion.changelog || '',
		dependencies: dependencies(prodVersion, options),
		release_channel: versionType(prodVersion),
		loaders: versionFields.loaders,
		featured: Boolean(prodVersion.featured),
		primary_file: primaryUpload.partName,
		status: options.versionStatus,
		file_types: fileTypes,
		ordering: prodVersion.ordering ?? null,
		...versionFields.fields,
	}

	const formData = new FormData()
	formData.append('data', JSON.stringify(createData))
	for (const file of uploadFiles) {
		formData.append(file.partName, file.blob, file.filename)
	}

	console.log(
		`  - ${prodVersion.version_number}: uploading ${uploadFiles.length} file(s) as ${createData.loaders.join(', ')}`,
	)
	return await requestJson(options.localApi, '/version', {
		method: 'POST',
		token: options.token,
		body: formData,
	})
}

async function downloadVersionFiles(prodFiles, options) {
	const usedNames = new Set()
	const files = []

	for (const [index, prodFile] of prodFiles.entries()) {
		if (prodFile.size && prodFile.size > options.maxFileBytes) {
			console.log(
				`    file skipped: ${prodFile.filename} (${formatBytes(prodFile.size)}, above limit)`,
			)
			continue
		}

		const response = await fetch(prodFile.url, {
			headers: { 'User-Agent': USER_AGENT },
		})
		if (!response.ok) {
			throw new Error(`download ${prodFile.url} failed with ${response.status}`)
		}

		const contentLength = Number(response.headers.get('content-length') || 0)
		if (contentLength > options.maxFileBytes) {
			console.log(
				`    file skipped: ${prodFile.filename} (${formatBytes(contentLength)}, above limit)`,
			)
			continue
		}

		const arrayBuffer = await response.arrayBuffer()
		if (arrayBuffer.byteLength > options.maxFileBytes) {
			console.log(
				`    file skipped: ${prodFile.filename} (${formatBytes(arrayBuffer.byteLength)}, above limit)`,
			)
			continue
		}

		const filename = uniqueFilename(
			safeFilename(prodFile.filename, prodFile.url, index),
			usedNames,
			index,
		)
		files.push({
			source: prodFile,
			filename,
			partName: `file_${index}`,
			primary: Boolean(prodFile.primary),
			blob: new Blob([arrayBuffer], {
				type: response.headers.get('content-type') || mimeFromFilename(filename),
			}),
		})
	}

	if (files.length === 0) {
		throw new SkipVersionError('all files were skipped')
	}
	return files
}

async function downloadIcon(iconUrl) {
	const response = await fetch(iconUrl, {
		headers: { 'User-Agent': USER_AGENT },
	})
	if (!response.ok) {
		throw new Error(`download ${iconUrl} failed with ${response.status}`)
	}
	const contentLength = Number(response.headers.get('content-length') || 0)
	if (contentLength > LOCAL_ICON_LIMIT_BYTES) {
		throw new Error(`icon is ${formatBytes(contentLength)}, above local 256 KiB limit`)
	}
	const arrayBuffer = await response.arrayBuffer()
	if (arrayBuffer.byteLength > LOCAL_ICON_LIMIT_BYTES) {
		throw new Error(`icon is ${formatBytes(arrayBuffer.byteLength)}, above local 256 KiB limit`)
	}

	const contentType = response.headers.get('content-type') || ''
	const filename = `icon.${extensionFromUrlOrType(iconUrl, contentType)}`
	return {
		filename,
		blob: new Blob([arrayBuffer], { type: contentType || mimeFromFilename(filename) }),
	}
}

function normalizeVersionFields(prodProject, prodVersion, primaryUpload, localMetadata) {
	const projectTypes = projectTypesFor(prodProject)
	const modpack = isModpack(projectTypes, primaryUpload.filename, prodVersion)
	let loaders = stringArray(prodVersion.loaders).filter((loader) => localMetadata.loaders.has(loader))
	const fields = {}

	if (modpack && localMetadata.loaders.has('mrpack')) {
		const mrpackLoaders = filterLoaderList(
			stringArray(fieldValue(prodVersion, 'mrpack_loaders')).length
				? stringArray(fieldValue(prodVersion, 'mrpack_loaders'))
				: loaders.filter((loader) => loader !== 'mrpack'),
			localMetadata,
		)
		loaders = ['mrpack']
		if (mrpackLoaders.length > 0) {
			fields.mrpack_loaders = mrpackLoaders
		} else if (localMetadata.loaders.has('fabric')) {
			fields.mrpack_loaders = ['fabric']
		}
	}

	if (loaders.length === 0) {
		loaders = fallbackLoaders(projectTypes, primaryUpload.filename, localMetadata)
	}

	const supportedFields = new Set(
		loaders.flatMap((loader) => localMetadata.loaders.get(loader)?.supported_fields || []),
	)

	if (supportedFields.has('game_versions')) {
		const gameVersions = filterGameVersions(fieldValue(prodVersion, 'game_versions'), localMetadata)
		fields.game_versions = gameVersions.length > 0 ? gameVersions : [localMetadata.fallbackGameVersion]
	}

	if (supportedFields.has('environment')) {
		const environment = fieldValue(prodVersion, 'environment')
		if (typeof environment === 'string' && localMetadata.environments.has(environment)) {
			fields.environment = environment
		} else if (localMetadata.environments.has('client_only_server_optional')) {
			fields.environment = 'client_only_server_optional'
		} else if (localMetadata.environments.size > 0) {
			fields.environment = [...localMetadata.environments][0]
		}
	}

	for (const [key, value] of Object.entries(extraLoaderFields(prodVersion))) {
		if (!supportedFields.has(key) || fields[key] !== undefined) continue
		fields[key] = value
	}

	return { loaders, fields }
}

function extraLoaderFields(version) {
	const fields = {}
	for (const [key, value] of Object.entries(version)) {
		if (!VERSION_CREATE_CORE_FIELDS.has(key) && value !== null && value !== undefined) {
			fields[key] = value
		}
	}
	if (version.fields && typeof version.fields === 'object') {
		for (const [key, value] of Object.entries(version.fields)) {
			if (value !== null && value !== undefined) fields[key] = value
		}
	}
	return fields
}

function fieldValue(object, key) {
	if (object && object[key] !== undefined) return object[key]
	if (object?.fields && object.fields[key] !== undefined) return object.fields[key]
	return undefined
}

function filterGameVersions(value, localMetadata) {
	return stringArray(value).filter((version) => localMetadata.gameVersions.has(version))
}

function filterLoaderList(value, localMetadata) {
	return stringArray(value).filter((loader) => loader !== 'mrpack' && localMetadata.loaders.has(loader))
}

function fallbackLoaders(projectTypes, filename, localMetadata) {
	const ext = extname(filename).toLowerCase()
	const candidates = []
	if (ext === '.mrpack' || projectTypes.includes('modpack')) candidates.push('mrpack')
	if (projectTypes.includes('resourcepack')) candidates.push('minecraft')
	if (projectTypes.includes('datapack')) candidates.push('datapack')
	if (projectTypes.includes('shader')) candidates.push('iris')
	if (projectTypes.includes('plugin')) candidates.push('paper')
	candidates.push('fabric', 'minecraft')

	for (const candidate of candidates) {
		if (localMetadata.loaders.has(candidate)) return [candidate]
	}
	return [[...localMetadata.loaders.keys()][0]].filter(Boolean)
}

function isModpack(projectTypes, filename, version) {
	return (
		projectTypes.includes('modpack') ||
		extname(filename).toLowerCase() === '.mrpack' ||
		stringArray(version.loaders).includes('mrpack') ||
		stringArray(fieldValue(version, 'mrpack_loaders')).length > 0
	)
}

function dependencies(version, options) {
	if (!options.includeDependencies) return []

	// Prod project/version ids do not exist locally unless separately mapped. Keep only
	// external file-name dependencies, which Labrinth can store without local id lookup.
	return (Array.isArray(version.dependencies) ? version.dependencies : [])
		.filter((dependency) => dependency.file_name && !dependency.project_id && !dependency.version_id)
		.map((dependency) => ({
			file_name: dependency.file_name,
			dependency_type: dependency.dependency_type || 'embedded',
		}))
}

function linkUrls(project, localMetadata) {
	const links = {}
	if (!localMetadata.linkPlatforms.size) return links

	if (project.link_urls && typeof project.link_urls === 'object') {
		for (const [name, value] of Object.entries(project.link_urls)) {
			const url = typeof value === 'string' ? value : value?.url
			if (url && localMetadata.linkPlatforms.has(name)) links[name] = url
		}
	}

	for (const [name, key] of [
		['issues', 'issues_url'],
		['source', 'source_url'],
		['wiki', 'wiki_url'],
		['discord', 'discord_url'],
	]) {
		if (project[key] && localMetadata.linkPlatforms.has(name)) {
			links[name] = project[key]
		}
	}

	return links
}

function filterCategories(categories, localMetadata, limit) {
	return unique(stringArray(categories))
		.filter((category) => localMetadata.categories.has(category))
		.slice(0, limit)
}

function projectTypesFor(project) {
	return unique([
		...stringArray(project.project_types),
		...stringArray(project.project_type),
		...stringArray(fieldValue(project, 'project_types')),
	])
}

function projectName(project) {
	return project.name || project.title || project.slug || project.id || 'Imported project'
}

function projectSummary(project) {
	return project.summary || (project.body ? project.description : null) || 'Imported from production Modrinth.'
}

function projectBody(project) {
	return project.body || (!project.body ? project.description : '') || ''
}

function licenseId(project) {
	const value = project.license?.id || project.license_id || project.license
	if (typeof value === 'string' && /^[A-Za-z0-9.+-]+$/.test(value)) return value
	return 'LicenseRef-All-Rights-Reserved'
}

function versionType(version) {
	const value = version.release_channel || version.version_type
	return ['release', 'beta', 'alpha'].includes(value) ? value : 'release'
}

async function requestJson(base, path, options = {}) {
	const method = options.method || 'GET'
	const url = buildUrl(base, path, options.query)
	const headers = {
		Accept: 'application/json',
		'User-Agent': USER_AGENT,
		...(options.headers || {}),
	}
	if (options.token) headers.Authorization = `Bearer ${options.token}`

	let body = options.body
	if (isJsonBody(body)) {
		headers['Content-Type'] = 'application/json'
		body = JSON.stringify(body)
	}

	const response = await fetch(url, { method, headers, body })
	const text = await response.text()
	const json = parseJson(text)

	if (!response.ok) {
		if (options.allow404 && response.status === 404) return null
		throw new HttpError(method, url.toString(), response.status, json || text)
	}

	return json ?? text
}

function buildUrl(base, path, query = {}) {
	const url = new URL(`${base.replace(/\/+$/, '')}/${path.replace(/^\/+/, '')}`)
	for (const [key, value] of Object.entries(query || {})) {
		if (value === null || value === undefined) continue
		url.searchParams.set(key, Array.isArray(value) ? JSON.stringify(value) : String(value))
	}
	return url
}

function parseJson(text) {
	if (!text) return null
	try {
		return JSON.parse(text)
	} catch {
		return null
	}
}

function isJsonBody(body) {
	return (
		body &&
		typeof body === 'object' &&
		!(body instanceof FormData) &&
		!(body instanceof Blob) &&
		!(body instanceof ArrayBuffer) &&
		!ArrayBuffer.isView(body) &&
		!(body instanceof URLSearchParams)
	)
}

function sanitizeSlug(value) {
	let slug = String(value || '')
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9!@$()`.+,_"-]/g, '-')
		.replace(/-+/g, '-')
		.replace(/^-+|-+$/g, '')
		.slice(0, 64)
	if (slug.length < 3) slug = `${slug || 'mod'}-import`.slice(0, 64)
	return slug
}

function sanitizeVersionNumber(value) {
	let version = String(value || 'imported')
		.trim()
		.replace(/[^a-zA-Z0-9!@$()`.+,_"-]/g, '_')
		.slice(0, 32)
	if (!version) version = 'imported'
	return version
}

function safeFilename(filename, url, index) {
	const candidate =
		filename ||
		decodeURIComponent(new URL(url).pathname.split('/').filter(Boolean).pop() || '') ||
		`file-${index}.jar`
	return basename(candidate).replaceAll('/', '_').replaceAll('\\', '_').replaceAll('\0', '_')
}

function uniqueFilename(filename, usedNames, index) {
	let candidate = filename
	let counter = index
	while (usedNames.has(candidate)) {
		const extension = extname(filename)
		const stem = extension ? filename.slice(0, -extension.length) : filename
		candidate = `${stem}-${counter}${extension}`
		counter += 1
	}
	usedNames.add(candidate)
	return candidate
}

function clampText(value, fallback, min, max) {
	const text = String(value || fallback).trim() || fallback
	const truncated = truncate(text, max)
	return truncated.length >= min ? truncated : fallback
}

function truncate(value, max) {
	const text = String(value || '')
	return text.length > max ? text.slice(0, max) : text
}

function stringArray(value) {
	if (Array.isArray(value)) {
		return value.filter((item) => typeof item === 'string' && item.length > 0)
	}
	if (typeof value === 'string' && value.length > 0) return [value]
	return []
}

function unique(values) {
	return [...new Set(values)]
}

function mimeFromFilename(filename) {
	switch (extname(filename).toLowerCase()) {
		case '.jar':
		case '.litemod':
			return 'application/java-archive'
		case '.zip':
		case '.mrpack':
			return 'application/zip'
		case '.png':
			return 'image/png'
		case '.jpg':
		case '.jpeg':
			return 'image/jpeg'
		case '.gif':
			return 'image/gif'
		case '.webp':
			return 'image/webp'
		case '.svg':
		case '.svgz':
			return 'image/svg+xml'
		default:
			return 'application/octet-stream'
	}
}

function extensionFromUrlOrType(url, contentType) {
	const extension = extname(new URL(url).pathname).replace(/^\./, '').toLowerCase()
	if (['png', 'jpg', 'jpeg', 'bmp', 'gif', 'webp', 'svg', 'svgz', 'rgb'].includes(extension)) {
		return extension
	}
	if (contentType.includes('png')) return 'png'
	if (contentType.includes('jpeg')) return 'jpg'
	if (contentType.includes('gif')) return 'gif'
	if (contentType.includes('webp')) return 'webp'
	if (contentType.includes('svg')) return 'svg'
	if (contentType.includes('bmp')) return 'bmp'
	return 'png'
}

function formatBytes(bytes) {
	if (!Number.isFinite(bytes)) return 'unknown size'
	const mib = bytes / (1024 * 1024)
	if (mib >= 1) return `${mib.toFixed(1)} MiB`
	return `${(bytes / 1024).toFixed(1)} KiB`
}

function sleep(ms) {
	return new Promise((resolveSleep) => setTimeout(resolveSleep, ms))
}

main().catch((error) => {
	console.error(error.message)
	process.exit(1)
})
