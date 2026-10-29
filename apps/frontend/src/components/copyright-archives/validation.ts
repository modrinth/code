import type { ArchiveInput } from './types'

function isObject(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function validateArchive(value: unknown): string[] {
	if (!isObject(value)) return ['Expected an archive object.']
	const errors: string[] = []
	function requiredString(object: Record<string, unknown>, key: string, prefix = '') {
		if (typeof object[key] !== 'string' || !object[key].trim()) {
			errors.push(`${prefix}${key}: enter a non-empty string.`)
		}
	}
	function allowedFields(object: Record<string, unknown>, fields: string[], prefix = '') {
		for (const key of Object.keys(object)) {
			if (!fields.includes(key)) errors.push(`${prefix}${key}: unknown field.`)
		}
	}
	allowedFields(value, [
		'id',
		'name',
		'modrinthMetadata',
		'minecraftMetadata',
		'license',
		'licenseTerms',
		'files',
	])
	if (value.id !== undefined) requiredString(value, 'id')
	requiredString(value, 'name')
	requiredString(value, 'license')
	if (value.modrinthMetadata !== undefined) {
		if (!isObject(value.modrinthMetadata)) errors.push('modrinthMetadata: expected an object.')
		else {
			allowedFields(
				value.modrinthMetadata,
				['projectId', 'versionId', 'projectType'],
				'modrinthMetadata.',
			)
			for (const key of ['projectId', 'versionId', 'projectType']) {
				requiredString(value.modrinthMetadata, key, 'modrinthMetadata.')
			}
		}
	}
	if (value.minecraftMetadata !== undefined && !isObject(value.minecraftMetadata)) {
		errors.push('minecraftMetadata: expected a JSON object.')
	}
	if (value.licenseTerms !== undefined) {
		if (!isObject(value.licenseTerms)) errors.push('licenseTerms: expected an object.')
		else {
			const terms = value.licenseTerms
			allowedFields(
				terms,
				[
					'requiresSource',
					'requiredRedistributionLicense',
					'redistributionAllowed',
					'commercialUseAllowed',
					'attributionRequired',
				],
				'licenseTerms.',
			)
			if (terms.requiresSource !== undefined && typeof terms.requiresSource !== 'string') {
				errors.push('licenseTerms.requiresSource: expected a string.')
			}
			for (const key of ['redistributionAllowed', 'commercialUseAllowed', 'attributionRequired']) {
				if (terms[key] !== undefined && typeof terms[key] !== 'boolean') {
					errors.push(`licenseTerms.${key}: expected true or false.`)
				}
			}
			if (
				terms.requiredRedistributionLicense !== undefined &&
				(!Array.isArray(terms.requiredRedistributionLicense) ||
					terms.requiredRedistributionLicense.some(
						(item) => typeof item !== 'string' || !item.trim(),
					))
			)
				errors.push(
					'licenseTerms.requiredRedistributionLicense: expected an array of license strings.',
				)
		}
	}
	if (!Array.isArray(value.files)) errors.push('files: expected an array.')
	else
		value.files.forEach((file: unknown, index: number) => {
			const prefix = `files[${index}].`
			if (!isObject(file)) {
				errors.push(`files[${index}]: expected an object.`)
				return
			}
			allowedFields(file, ['path', 'filename', 'hash_sha512'], prefix)
			if (typeof file.path !== 'string') errors.push(`${prefix}path: expected a string.`)
			requiredString(file, 'filename', prefix)
			if (typeof file.hash_sha512 !== 'string' || !/^[a-f\d]{128}$/i.test(file.hash_sha512)) {
				errors.push(`${prefix}hash_sha512: expected 128 hexadecimal characters.`)
			}
		})
	return errors
}

export function parseImport(json: string, existingIds: Set<string>): ArchiveInput[] {
	const value: unknown = JSON.parse(json)
	if (!Array.isArray(value) || !value.length)
		throw new Error('Enter a non-empty JSON array of archives.')
	const errors: string[] = []
	const ids = new Set<string>()
	value.forEach((entry: unknown, index: number) => {
		for (const error of validateArchive(entry)) errors.push(`Archive ${index + 1}: ${error}`)
		if (isObject(entry) && typeof entry.id === 'string') {
			if (ids.has(entry.id)) errors.push(`Archive ${index + 1}: duplicate ID "${entry.id}".`)
			if (!existingIds.has(entry.id))
				errors.push(
					`Archive ${index + 1}: unknown ID "${entry.id}". Omit the ID to create an archive.`,
				)
			ids.add(entry.id)
		}
	})
	if (errors.length) throw new Error(errors.join('\n'))
	return value as ArchiveInput[]
}
