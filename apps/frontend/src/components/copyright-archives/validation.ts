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
		'modrinth_metadata',
		'minecraft_metadata',
		'license',
		'license_terms',
		'files',
	])
	if (value.id !== undefined) requiredString(value, 'id')
	requiredString(value, 'name')
	requiredString(value, 'license')
	if (value.modrinth_metadata != null) {
		if (!isObject(value.modrinth_metadata)) errors.push('modrinth_metadata: expected an object.')
		else {
			allowedFields(
				value.modrinth_metadata,
				['project_id', 'version_id', 'project_type'],
				'modrinth_metadata.',
			)
			for (const key of ['project_id', 'version_id', 'project_type']) {
				requiredString(value.modrinth_metadata, key, 'modrinth_metadata.')
			}
		}
	}
	if (value.minecraft_metadata != null) {
		if (!isObject(value.minecraft_metadata)) errors.push('minecraft_metadata: expected an object.')
		else {
			allowedFields(value.minecraft_metadata, ['version'], 'minecraft_metadata.')
			requiredString(value.minecraft_metadata, 'version', 'minecraft_metadata.')
		}
	}
	if (value.license_terms != null) {
		if (!isObject(value.license_terms)) errors.push('license_terms: expected an object.')
		else {
			const terms = value.license_terms
			allowedFields(
				terms,
				[
					'requires_source',
					'required_redistribution_license',
					'redistribution_allowed',
					'commercial_use_allowed',
					'attribution_required',
				],
				'license_terms.',
			)
			if (terms.requires_source != null && typeof terms.requires_source !== 'string') {
				errors.push('license_terms.requires_source: expected a string.')
			}
			for (const key of [
				'redistribution_allowed',
				'commercial_use_allowed',
				'attribution_required',
			]) {
				if (terms[key] != null && typeof terms[key] !== 'boolean') {
					errors.push(`license_terms.${key}: expected true or false.`)
				}
			}
			if (
				terms.required_redistribution_license != null &&
				(!Array.isArray(terms.required_redistribution_license) ||
					terms.required_redistribution_license.some(
						(item) => typeof item !== 'string' || !item.trim(),
					))
			)
				errors.push(
					'license_terms.required_redistribution_license: expected an array of license strings.',
				)
		}
	}
	if (!Array.isArray(value.files))
		errors.push(
			'files: provide the complete file list to replace an archive; the API does not return it.',
		)
	else
		value.files.forEach((file: unknown, index: number) => {
			const prefix = `files[${index}].`
			if (!isObject(file)) {
				errors.push(`files[${index}]: expected an object.`)
				return
			}
			allowedFields(file, ['path', 'sha512'], prefix)
			if (typeof file.path !== 'string') errors.push(`${prefix}path: expected a string.`)
			if (typeof file.sha512 !== 'string' || !/^[a-f\d]{128}$/i.test(file.sha512)) {
				errors.push(`${prefix}sha512: expected 128 hexadecimal characters.`)
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
