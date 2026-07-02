import chalk from 'chalk'
import * as fs from 'node:fs'
import * as path from 'node:path'
import { fileURLToPath } from 'node:url'

const HTTP_METHODS = ['get', 'put', 'post', 'delete', 'patch', 'head', 'options', 'trace'] as const
const BODY_EXTRACTORS = ['json', 'form', 'payload', 'bytes', 'multipart', 'text'] as const
const AUTH_HEADERS = {
	authorization: 'Bearer mra_admin',
	'modrinth-admin': 'feedbeef',
}
const IGNORED_PROBE_OPERATIONS = new Set([
	'POST /v2/admin/_force_reindex',
	'POST /v2/admin/_force_reindex/{project_id}',
])

type HttpMethod = (typeof HTTP_METHODS)[number]
type BodyExtractor = (typeof BODY_EXTRACTORS)[number]

interface Args {
	baseUrl: string
	openapi?: string
	sourceDir: string
	probe: boolean
	staticCheck: boolean
	includeMutatingProbes: boolean
	json: boolean
	verbose: boolean
}

interface OpenApiSpec {
	paths?: Record<string, Record<string, OpenApiOperation>>
	components?: unknown
}

interface OpenApiOperation {
	operationId?: string
	parameters?: OpenApiParameter[]
	requestBody?: OpenApiRequestBody | { $ref: string }
	responses?: unknown
	[key: string]: unknown
}

interface OpenApiParameter {
	name?: string
	in?: string
	required?: boolean
	schema?: OpenApiSchema
}

interface OpenApiRequestBody {
	required?: boolean
	content?: Record<string, { schema?: OpenApiSchema }>
}

interface OpenApiSchema {
	type?: string | string[]
	format?: string
	$ref?: string
	oneOf?: OpenApiSchema[]
	anyOf?: OpenApiSchema[]
	allOf?: OpenApiSchema[]
}

interface Operation {
	path: string
	method: HttpMethod
	operationId: string
	operation: OpenApiOperation
	pathParams: Set<string>
	queryParams: Set<string>
	requestBodyContentTypes: Set<string>
	hasRequestBody: boolean
}

interface StructField {
	name: string
	type: string
	optional: boolean
}

interface StructInfo {
	name: string
	file: string
	fields: StructField[]
}

interface HandlerInfo {
	operationId: string
	functionName: string
	file: string
	line: number
	importedTypes: Map<string, string>
	jsonTypes: string[]
	queryTypes: string[]
	pathTypes: string[]
	formTypes: string[]
	bodyExtractors: Set<BodyExtractor>
	routeMethods: Set<HttpMethod>
	routePathParams: Set<string>
}

interface SourceExpectations {
	handlers: HandlerInfo[]
	structsByFileAndName: Map<string, StructInfo>
	structsByName: Map<string, StructInfo[]>
}

interface Issue {
	severity: 'error' | 'warning'
	code: string
	message: string
	file?: string
	line?: number
	operation?: string
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

function parseArgs(): Args {
	const args = process.argv.slice(2)
	const parsed: Args = {
		baseUrl: 'http://127.0.0.1:8000',
		sourceDir: path.join(repoRoot, 'apps/labrinth/src'),
		probe: true,
		staticCheck: true,
		includeMutatingProbes: true,
		json: false,
		verbose: false,
	}

	for (let i = 0; i < args.length; i++) {
		const arg = args[i]
		const next = () => {
			const value = args[++i]
			if (!value) throw new Error(`missing value for ${arg}`)
			return value
		}

		switch (arg) {
			case '--base-url':
				parsed.baseUrl = next().replace(/\/+$/, '')
				break
			case '--openapi':
				parsed.openapi = next()
				break
			case '--source':
				parsed.sourceDir = path.resolve(next())
				break
			case '--no-probe':
				parsed.probe = false
				break
			case '--no-static':
				parsed.staticCheck = false
				break
			case '--safe-probes':
				parsed.includeMutatingProbes = false
				break
			case '--include-mutating-probes':
				parsed.includeMutatingProbes = true
				break
			case '--json':
				parsed.json = true
				break
			case '--verbose':
				parsed.verbose = true
				break
			case '--help':
			case '-h':
				printUsage()
				process.exit(0)
			default:
				throw new Error(`unknown argument ${arg}`)
		}
	}

	return parsed
}

function printUsage() {
	console.log(`Usage: pnpm scripts labrinth-openapi-check [options]

Options:
  --base-url <url>    Running Labrinth URL. Default: http://127.0.0.1:8000
  --openapi <path>    OpenAPI JSON file or URL. Default: <base-url>/docs
  --source <path>     Labrinth source directory. Default: apps/labrinth/src
  --no-probe          Skip live route reachability checks
  --no-static         Skip Rust handler signature checks
  --safe-probes       Only probe GET/HEAD routes
  --include-mutating-probes
                      Probe non-GET/HEAD routes. Enabled by default.
  --json              Print machine-readable JSON
  --verbose           Print skipped static comparisons`)
}

async function main() {
	const args = parseArgs()
	const spec = await loadSpec(args)
	const operations = collectOperations(spec)
	const issues: Issue[] = []

	if (args.probe) {
		issues.push(...(await probeOperations(args.baseUrl, operations, args.includeMutatingProbes)))
	}

	if (args.staticCheck) {
		const source = collectSourceExpectations(args.sourceDir)
		issues.push(...compareSourceToSpec(operations, source, args.verbose))
	}

	if (args.json) {
		console.log(JSON.stringify({ operations: operations.length, issues }, null, 2))
	} else {
		printReport(operations, issues)
	}

	if (issues.some((issue) => issue.severity === 'error')) {
		process.exit(1)
	}
}

async function loadSpec(args: Args): Promise<OpenApiSpec> {
	const source = args.openapi ?? `${args.baseUrl}/docs`
	const text = await readText(source)
	const trimmed = text.trim()

	if (trimmed.startsWith('{')) {
		return JSON.parse(trimmed)
	}

	const sourceUrls = [...trimmed.matchAll(/"url"\s*:\s*"([^"]+\.json)"/g)].map((match) => match[1])
	if (sourceUrls.length) {
		const specs = await Promise.all(
			sourceUrls.map(async (url) => {
				const resolved = resolveSource(url, source)
				const specText = await readText(resolved)
				return JSON.parse(specText) as OpenApiSpec
			}),
		)

		return mergeSpecs(specs)
	}

	const scalarScript = trimmed.match(
		/<script\b[^>]*\bid=(["'])api-reference\1[^>]*>([\s\S]*?)<\/script>/i,
	)
	if (!scalarScript) {
		throw new Error(`could not find OpenAPI JSON in ${source}`)
	}

	return JSON.parse(decodeHtmlEntities(scalarScript[2].trim()))
}

function resolveSource(source: string, baseSource: string): string {
	if (/^https?:\/\//.test(source)) return source

	if (/^https?:\/\//.test(baseSource)) {
		return new URL(source, baseSource).toString()
	}

	return path.resolve(path.dirname(path.resolve(baseSource)), source)
}

function mergeSpecs(specs: OpenApiSpec[]): OpenApiSpec {
	const merged: OpenApiSpec = { paths: {} }

	for (const spec of specs) {
		Object.assign(merged.paths ?? {}, spec.paths ?? {})
	}

	return merged
}

async function readText(source: string): Promise<string> {
	if (/^https?:\/\//.test(source)) {
		const response = await fetch(source, { headers: AUTH_HEADERS })
		if (!response.ok) {
			throw new Error(`failed to fetch ${source}: HTTP ${response.status}`)
		}
		return response.text()
	}

	return fs.readFileSync(path.resolve(source), 'utf8')
}

function decodeHtmlEntities(value: string): string {
	return value
		.replace(/&quot;/g, '"')
		.replace(/&#34;/g, '"')
		.replace(/&apos;/g, "'")
		.replace(/&#39;/g, "'")
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&amp;/g, '&')
}

function collectOperations(spec: OpenApiSpec): Operation[] {
	const operations: Operation[] = []

	for (const [apiPath, pathItem] of Object.entries(spec.paths ?? {})) {
		for (const method of HTTP_METHODS) {
			const operation = pathItem[method]
			if (!operation) continue

			const parameters = operation.parameters ?? []
			const requestBody =
				operation.requestBody && !('$ref' in operation.requestBody)
					? operation.requestBody
					: undefined

			operations.push({
				path: apiPath,
				method,
				operationId: operation.operationId ?? `${method} ${apiPath}`,
				operation,
				pathParams: new Set(
					parameters
						.filter((param) => param.in === 'path' && param.name)
						.map((param) => param.name as string),
				),
				queryParams: new Set(
					parameters
						.filter((param) => param.in === 'query' && param.name)
						.map((param) => param.name as string),
				),
				requestBodyContentTypes: new Set(Object.keys(requestBody?.content ?? {})),
				hasRequestBody: Boolean(requestBody),
			})
		}
	}

	return operations.sort((a, b) => `${a.path} ${a.method}`.localeCompare(`${b.path} ${b.method}`))
}

async function probeOperations(
	baseUrl: string,
	operations: Operation[],
	includeMutatingProbes: boolean,
): Promise<Issue[]> {
	const issues: Issue[] = []

	for (const operation of operations) {
		if (IGNORED_PROBE_OPERATIONS.has(operationKey(operation))) {
			continue
		}

		if (!includeMutatingProbes && !['get', 'head'].includes(operation.method)) {
			continue
		}

		const url = new URL(renderProbePath(operation), `${baseUrl}/`)
		const { body, contentType } = bodyForProbe(operation)
		const headers: Record<string, string> = {
			...AUTH_HEADERS,
			accept: 'application/json',
			'user-agent': 'modrinth-labrinth-openapi-check',
		}
		if (contentType) headers['content-type'] = contentType

		let response: Response
		let text = ''
		try {
			response = await fetch(url, {
				method: operation.method.toUpperCase(),
				headers,
				body,
				redirect: 'manual',
			})
			text = await response.text()
		} catch (error) {
			issues.push({
				severity: 'error',
				code: 'probe_failed',
				operation: formatOperation(operation),
				message: `could not probe ${url}: ${String(error)}`,
			})
			continue
		}

		if (isActixMissingRoute(response, text)) {
			issues.push({
				severity: 'error',
				code: 'documented_route_unreachable',
				operation: formatOperation(operation),
				message: `${operation.method.toUpperCase()} ${operation.path} is in OpenAPI but the running backend returned the fallback 404`,
			})
		}
	}

	return issues
}

function renderProbePath(operation: Operation): string {
	return operation.path.replace(/\{([^}]+)\}/g, (_, name: string) =>
		encodeURIComponent(samplePathValue(name, operation)),
	)
}

function samplePathValue(name: string, operation: Operation): string {
	const param = (operation.operation.parameters ?? []).find(
		(candidate) => candidate.in === 'path' && candidate.name === name,
	)
	const schema = param?.schema
	const ref = schema?.$ref?.split('/').at(-1)?.toLowerCase() ?? ''
	const type = Array.isArray(schema?.type) ? schema?.type.join('|') : schema?.type

	if (name.toLowerCase().includes('sha1')) return '0000000000000000000000000000000000000000'
	if (schema?.format === 'uuid') return '00000000-0000-0000-0000-000000000000'
	if (type === 'integer' || ref.endsWith('id') || name.toLowerCase().endsWith('id')) return '1'
	return 'test'
}

function bodyForProbe(operation: Operation): { body?: BodyInit; contentType?: string } {
	if (!operation.hasRequestBody) return {}

	if (operation.requestBodyContentTypes.has('application/json')) {
		return { body: '{}', contentType: 'application/json' }
	}
	if (operation.requestBodyContentTypes.has('text/plain')) {
		return { body: 'test', contentType: 'text/plain' }
	}
	if (
		[...operation.requestBodyContentTypes].some((type) => type.startsWith('multipart/form-data'))
	) {
		return { body: '', contentType: 'multipart/form-data; boundary=openapi-check' }
	}

	const first = [...operation.requestBodyContentTypes][0]
	return { body: '', contentType: first }
}

function isActixMissingRoute(response: Response, text: string): boolean {
	if (response.status !== 404) return false

	try {
		const parsed = JSON.parse(text)
		return (
			parsed?.error === 'not_found' && parsed?.description === 'the requested route does not exist'
		)
	} catch {
		return false
	}
}

function collectSourceExpectations(sourceDir: string): SourceExpectations {
	const files = findRustFiles(sourceDir)
	const structsByFileAndName = new Map<string, StructInfo>()
	const structsByName = new Map<string, StructInfo[]>()
	const handlers: HandlerInfo[] = []

	for (const file of files) {
		const content = fs.readFileSync(file, 'utf8')
		for (const structInfo of extractStructs(content, file)) {
			structsByFileAndName.set(structKey(file, structInfo.name), structInfo)
			const namedStructs = structsByName.get(structInfo.name) ?? []
			namedStructs.push(structInfo)
			structsByName.set(structInfo.name, namedStructs)
		}
		handlers.push(...extractHandlers(content, file, sourceDir))
	}

	return { handlers, structsByFileAndName, structsByName }
}

function findRustFiles(root: string): string[] {
	const files: string[] = []

	function walk(current: string) {
		for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
			const fullPath = path.join(current, entry.name)
			if (entry.isDirectory()) {
				if (!entry.name.startsWith('.') && entry.name !== 'target') walk(fullPath)
			} else if (entry.isFile() && entry.name.endsWith('.rs')) {
				files.push(fullPath)
			}
		}
	}

	walk(root)
	return files
}

function extractStructs(content: string, file: string): StructInfo[] {
	const structs: StructInfo[] = []
	const regex =
		/((?:\s*#\[[\s\S]*?\]\s*)*)(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/g
	let match: RegExpExecArray | null

	while ((match = regex.exec(content))) {
		const name = match[2]
		const openBrace = content.indexOf('{', match.index + match[0].length - 1)
		const closeBrace = findMatching(content, openBrace, '{', '}')
		if (closeBrace === -1) continue

		const attrs = match[1]
		const body = content.slice(openBrace + 1, closeBrace)
		const renameAll = extractSerdeRenameAll(attrs)
		const fields: StructField[] = []
		const fieldRegex =
			/((?:\s*#\[[^\]]*\]\s*)*)\s*pub(?:\([^)]*\))?\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,\n]+(?:<[^;{}]*>)?)/g
		let fieldMatch: RegExpExecArray | null

		while ((fieldMatch = fieldRegex.exec(body))) {
			const fieldAttrs = fieldMatch[1]
			if (/\bskip_deserializing\b|\bskip\b/.test(fieldAttrs)) continue

			const rustName = fieldMatch[2]
			const rustType = fieldMatch[3].trim()
			fields.push({
				name: extractSerdeRename(fieldAttrs) ?? applyRenameAll(rustName, renameAll),
				type: rustType,
				optional: /^Option\s*</.test(rustType) || /\bdefault\b/.test(fieldAttrs),
			})
		}

		structs.push({ name, file, fields })
		regex.lastIndex = closeBrace + 1
	}

	return structs
}

function extractHandlers(content: string, file: string, sourceDir: string): HandlerInfo[] {
	const handlers: HandlerInfo[] = []
	const importedTypes = extractImportedTypeFiles(content, sourceDir)
	let index = 0

	while (true) {
		const attrStart = content.indexOf('#[utoipa::path', index)
		if (attrStart === -1) break

		const attrOpen = content.indexOf('[', attrStart)
		const attrClose = findMatching(content, attrOpen, '[', ']')
		if (attrClose === -1) break

		const attrText = content.slice(attrStart, attrClose + 1)
		const rest = content.slice(attrClose + 1)
		const fnMatch = rest.match(
			/^\s*(?:#\[[\s\S]*?\]\s*)*(?:pub(?:\([^)]*\))?\s+)?async\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/,
		)
		if (!fnMatch?.index && fnMatch?.index !== 0) {
			index = attrClose + 1
			continue
		}

		const fnStart = attrClose + 1 + fnMatch.index
		const fnName = fnMatch[1]
		const parenStart = content.indexOf('(', fnStart + fnMatch[0].lastIndexOf(fnName))
		const parenEnd = findMatching(content, parenStart, '(', ')')
		if (parenEnd === -1) {
			index = attrClose + 1
			continue
		}

		const betweenAttrs = content.slice(attrClose + 1, fnStart)
		const params = content.slice(parenStart + 1, parenEnd)
		const routeAttrs = [
			...betweenAttrs.matchAll(
				/#\[(get|put|post|delete|patch|head|options|trace|route)\s*\(([\s\S]*?)\)\]/g,
			),
		]
		const routeMethods = new Set<HttpMethod>()
		const routePathParams = new Set<string>()

		for (const routeAttr of routeAttrs) {
			const method = routeAttr[1]
			const body = routeAttr[2]
			if (method === 'route') {
				for (const methodMatch of body.matchAll(/method\s*=\s*"([A-Z]+)"/g)) {
					const routeMethod = methodMatch[1].toLowerCase() as HttpMethod
					if (HTTP_METHODS.includes(routeMethod)) routeMethods.add(routeMethod)
				}
			} else {
				routeMethods.add(method as HttpMethod)
			}

			const pathMatch = body.match(/"([^"]+)"/)
			if (pathMatch) {
				for (const paramMatch of pathMatch[1].matchAll(/\{([^}:]+)(?::[^}]+)?\}/g)) {
					routePathParams.add(paramMatch[1])
				}
			}
		}

		const operationId = attrText.match(/\boperation_id\s*=\s*"([^"]+)"/)?.[1] ?? fnName
		const jsonTypes = extractGenericExtractorTypes(params, 'Json')
		const queryTypes = extractGenericExtractorTypes(params, 'Query')
		const pathTypes = extractGenericExtractorTypes(params, 'Path')
		const formTypes = extractGenericExtractorTypes(params, 'Form')
		const bodyExtractors = new Set<BodyExtractor>()

		if (jsonTypes.length) bodyExtractors.add('json')
		if (formTypes.length) bodyExtractors.add('form')
		if (/\b(?:web::)?Payload\b/.test(params)) bodyExtractors.add('payload')
		if (/\b(?:web::)?Bytes\b/.test(params)) bodyExtractors.add('bytes')
		if (/\bMultipart\b/.test(params)) bodyExtractors.add('multipart')
		if (/(?:^|,)\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*:\s*)?String\s*(?:,|$)/.test(params)) {
			bodyExtractors.add('text')
		}

		handlers.push({
			operationId,
			functionName: fnName,
			file,
			line: lineNumber(content, attrStart),
			importedTypes,
			jsonTypes,
			queryTypes,
			pathTypes,
			formTypes,
			bodyExtractors,
			routeMethods,
			routePathParams,
		})

		index = parenEnd + 1
	}

	return handlers
}

function extractImportedTypeFiles(content: string, sourceDir: string): Map<string, string> {
	const importedTypes = new Map<string, string>()

	for (const importMatch of content.matchAll(/use\s+crate::([\w:]+)::\{([^}]+)\};/g)) {
		const moduleFile = resolveModuleFile(sourceDir, importMatch[1])
		if (!moduleFile) continue

		for (const rawItem of importMatch[2].split(',')) {
			const item = rawItem.trim()
			if (!item || item === 'self' || item.includes('::')) continue

			const simpleName = item.match(/^([A-Za-z_][A-Za-z0-9_]*)(?:\s+as\s+\w+)?$/)?.[1]
			if (simpleName) importedTypes.set(simpleName, moduleFile)
		}
	}

	for (const importMatch of content.matchAll(
		/use\s+crate::([\w:]+)::([A-Za-z_][A-Za-z0-9_]*)(?:\s+as\s+\w+)?;/g,
	)) {
		const moduleFile = resolveModuleFile(sourceDir, importMatch[1])
		if (moduleFile) importedTypes.set(importMatch[2], moduleFile)
	}

	return importedTypes
}

function resolveModuleFile(sourceDir: string, modulePath: string): string | null {
	const relativePath = modulePath.replace(/::/g, path.sep)
	const directFile = path.join(sourceDir, `${relativePath}.rs`)
	if (fs.existsSync(directFile)) return directFile

	const modFile = path.join(sourceDir, relativePath, 'mod.rs')
	if (fs.existsSync(modFile)) return modFile

	return null
}

function extractGenericExtractorTypes(params: string, extractor: string): string[] {
	const types: string[] = []
	const regex = new RegExp(`(?:web::)?${extractor}\\s*<`, 'g')
	let match: RegExpExecArray | null

	while ((match = regex.exec(params))) {
		const open = params.indexOf('<', match.index)
		const close = findMatching(params, open, '<', '>')
		if (close === -1) continue

		types.push(params.slice(open + 1, close).trim())
		regex.lastIndex = close + 1
	}

	return types
}

function compareSourceToSpec(
	operations: Operation[],
	source: SourceExpectations,
	verbose: boolean,
): Issue[] {
	const issues: Issue[] = []
	const operationsById = new Map<string, Operation[]>()

	for (const operation of operations) {
		const existing = operationsById.get(operation.operationId) ?? []
		existing.push(operation)
		operationsById.set(operation.operationId, existing)
	}

	for (const handler of source.handlers) {
		const candidates = operationsById.get(handler.operationId) ?? []
		if (!candidates.length) {
			issues.push({
				severity: 'error',
				code: 'annotated_handler_missing_from_openapi',
				file: handler.file,
				line: handler.line,
				operation: handler.operationId,
				message: `handler ${handler.functionName} has #[utoipa::path] but no OpenAPI operation with operationId ${handler.operationId}`,
			})
			continue
		}

		const operation = chooseOperation(handler, candidates)
		if (!operation) {
			if (verbose) {
				issues.push({
					severity: 'warning',
					code: 'duplicate_operation_id_skipped',
					file: handler.file,
					line: handler.line,
					operation: handler.operationId,
					message: `operationId ${handler.operationId} appears ${candidates.length} times, so static signature comparison was skipped`,
				})
			}
			continue
		}

		compareBody(handler, operation, issues)
		compareQuery(handler, operation, source, issues)
		comparePath(handler, operation, issues)
	}

	return issues
}

function chooseOperation(handler: HandlerInfo, candidates: Operation[]): Operation | null {
	if (candidates.length === 1) return candidates[0]

	const methodMatches = candidates.filter(
		(operation) => !handler.routeMethods.size || handler.routeMethods.has(operation.method),
	)
	if (methodMatches.length === 1) return methodMatches[0]

	const pathMatches = methodMatches.filter((operation) => {
		for (const param of handler.routePathParams) {
			if (!operation.pathParams.has(param)) return false
		}
		return true
	})
	if (pathMatches.length === 1) return pathMatches[0]

	return null
}

function compareBody(handler: HandlerInfo, operation: Operation, issues: Issue[]) {
	const expectsBody = handler.bodyExtractors.size > 0

	if (expectsBody && !operation.hasRequestBody) {
		issues.push({
			severity: 'error',
			code: 'missing_request_body',
			file: handler.file,
			line: handler.line,
			operation: formatOperation(operation),
			message: `${handler.functionName} expects ${[...handler.bodyExtractors].join(', ')} body data, but OpenAPI has no requestBody`,
		})
	}

	if (!expectsBody && operation.hasRequestBody) {
		issues.push({
			severity: 'error',
			code: 'unexpected_request_body',
			file: handler.file,
			line: handler.line,
			operation: formatOperation(operation),
			message: `OpenAPI documents a requestBody, but ${handler.functionName} has no recognized body extractor`,
		})
	}

	if (
		handler.bodyExtractors.has('json') &&
		!operation.requestBodyContentTypes.has('application/json')
	) {
		issues.push({
			severity: 'error',
			code: 'request_body_content_type_mismatch',
			file: handler.file,
			line: handler.line,
			operation: formatOperation(operation),
			message: `${handler.functionName} expects web::Json, but OpenAPI content types are ${formatSet(operation.requestBodyContentTypes)}`,
		})
	}

	if (handler.bodyExtractors.has('text') && !operation.requestBodyContentTypes.has('text/plain')) {
		issues.push({
			severity: 'error',
			code: 'request_body_content_type_mismatch',
			file: handler.file,
			line: handler.line,
			operation: formatOperation(operation),
			message: `${handler.functionName} expects a plain String body, but OpenAPI content types are ${formatSet(operation.requestBodyContentTypes)}`,
		})
	}
}

function compareQuery(
	handler: HandlerInfo,
	operation: Operation,
	source: SourceExpectations,
	issues: Issue[],
) {
	const expectedQueryParams = new Set<string>()

	for (const queryType of handler.queryTypes) {
		const simpleType = simpleRustType(queryType)
		if (isDynamicExtractorType(simpleType)) continue

		const structInfo = resolveStructInfo(handler, simpleType, source)
		if (!structInfo) {
			issues.push({
				severity: 'warning',
				code: 'query_type_not_resolved',
				file: handler.file,
				line: handler.line,
				operation: formatOperation(operation),
				message: `could not resolve query type ${queryType} for ${handler.functionName}`,
			})
			continue
		}

		for (const field of structInfo.fields) expectedQueryParams.add(field.name)
	}

	for (const expected of expectedQueryParams) {
		if (!operation.queryParams.has(expected)) {
			issues.push({
				severity: 'error',
				code: 'missing_query_parameter',
				file: handler.file,
				line: handler.line,
				operation: formatOperation(operation),
				message: `${handler.functionName} expects query parameter ${expected}, but OpenAPI does not document it`,
			})
		}
	}
}

function resolveStructInfo(
	handler: HandlerInfo,
	simpleType: string,
	source: SourceExpectations,
): StructInfo | null {
	const sameFileStruct = source.structsByFileAndName.get(structKey(handler.file, simpleType))
	if (sameFileStruct) return sameFileStruct

	const importedFile = handler.importedTypes.get(simpleType)
	if (importedFile) {
		const importedStruct = source.structsByFileAndName.get(structKey(importedFile, simpleType))
		if (importedStruct) return importedStruct
	}

	const structs = source.structsByName.get(simpleType) ?? []
	if (structs.length === 1) return structs[0]

	return null
}

function comparePath(handler: HandlerInfo, operation: Operation, issues: Issue[]) {
	if (handler.pathTypes.length && !operation.pathParams.size) {
		issues.push({
			severity: 'error',
			code: 'missing_path_parameters',
			file: handler.file,
			line: handler.line,
			operation: formatOperation(operation),
			message: `${handler.functionName} extracts path data, but OpenAPI has no path parameters`,
		})
	}

	for (const routeParam of handler.routePathParams) {
		if (!operation.pathParams.has(routeParam)) {
			issues.push({
				severity: 'error',
				code: 'path_parameter_name_mismatch',
				file: handler.file,
				line: handler.line,
				operation: formatOperation(operation),
				message: `${handler.functionName} route macro uses path parameter ${routeParam}, but OpenAPI path parameters are ${formatSet(operation.pathParams)}`,
			})
		}
	}
}

function simpleRustType(type: string): string {
	const trimmed = type.trim()
	if (trimmed.startsWith('Option<')) {
		return simpleRustType(trimmed.slice('Option<'.length, -1))
	}

	const withoutGenerics = trimmed.replace(/<[\s\S]*>$/, '')
	return withoutGenerics.split('::').at(-1)?.replace(/[^\w]/g, '') ?? withoutGenerics
}

function structKey(file: string, name: string): string {
	return `${file}::${name}`
}

function isDynamicExtractorType(type: string): boolean {
	return ['HashMap', 'Value', 'Map'].includes(type)
}

function extractSerdeRename(attrs: string): string | undefined {
	return attrs.match(/\brename\s*=\s*"([^"]+)"/)?.[1]
}

function extractSerdeRenameAll(attrs: string): string | undefined {
	return attrs.match(/\brename_all\s*=\s*"([^"]+)"/)?.[1]
}

function applyRenameAll(name: string, renameAll?: string): string {
	switch (renameAll) {
		case 'camelCase':
			return name.replace(/_([a-z])/g, (_, char: string) => char.toUpperCase())
		case 'kebab-case':
			return name.replace(/_/g, '-')
		default:
			return name
	}
}

function findMatching(content: string, openIndex: number, open: string, close: string): number {
	if (openIndex < 0 || content[openIndex] !== open) return -1

	let depth = 0
	let quote: string | null = null
	let escaped = false

	for (let i = openIndex; i < content.length; i++) {
		const char = content[i]

		if (quote) {
			if (escaped) {
				escaped = false
			} else if (char === '\\') {
				escaped = true
			} else if (char === quote) {
				quote = null
			}
			continue
		}

		if (char === '"') {
			quote = char
			continue
		}

		if (char === open) depth++
		if (char === close) {
			depth--
			if (depth === 0) return i
		}
	}

	return -1
}

function lineNumber(content: string, index: number): number {
	let line = 1
	for (let i = 0; i < index; i++) {
		if (content[i] === '\n') line++
	}
	return line
}

function printReport(operations: Operation[], issues: Issue[]) {
	const errors = issues.filter((issue) => issue.severity === 'error')
	const warnings = issues.filter((issue) => issue.severity === 'warning')

	console.log(chalk.bold(`Checked ${operations.length} OpenAPI operations`))

	if (!issues.length) {
		console.log(chalk.green('No OpenAPI mismatches found'))
		return
	}

	for (const issue of issues) {
		const color = issue.severity === 'error' ? chalk.red : chalk.yellow
		const location = issue.file ? ` ${formatLocation(issue.file, issue.line)}` : ''
		const operation = issue.operation ? ` ${chalk.gray(issue.operation)}` : ''
		console.log(
			`${color(issue.severity.toUpperCase())} ${chalk.bold(issue.code)}${operation}${location}`,
		)
		console.log(`  ${issue.message}`)
	}

	console.log()
	console.log(
		`${chalk.red(`${errors.length} error(s)`)} ${chalk.yellow(`${warnings.length} warning(s)`)}`,
	)
}

function formatOperation(operation: Operation): string {
	return `${operation.method.toUpperCase()} ${operation.path} (${operation.operationId})`
}

function operationKey(operation: Operation): string {
	return `${operation.method.toUpperCase()} ${operation.path}`
}

function formatSet(values: Set<string>): string {
	return values.size ? [...values].join(', ') : '<none>'
}

function formatLocation(file: string, line?: number): string {
	const relative = path.relative(repoRoot, file)
	return line ? `${relative}:${line}` : relative
}

main().catch((error) => {
	console.error(chalk.red(error instanceof Error ? error.message : String(error)))
	process.exit(1)
})
