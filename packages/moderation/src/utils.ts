import type { Labrinth } from '@modrinth/api-client'

export function expandVariables(
	template: string,
	project: Labrinth.Projects.v2.Project,
	projectV3: Labrinth.Projects.v3.Project,
	variables?: Record<string, string>,
): string {
	variables ??= {
		...flattenStaticVariables(),
		...flattenProjectVariables(project),
		...flattenProjectV3Variables(projectV3),
	}

	return Object.entries(variables).reduce((result, [key, value]) => {
		const variable = `%${key}%`
		return result.replace(new RegExp(variable, 'g'), value)
	}, template)
}

export const licensesNotRequiringSource: string[] = [
	'LicenseRef-All-Rights-Reserved',
	'Apache-2.0',
	'BSD-2-Clause',
	'BSD-3-Clause',
	'CC0-1.0',
	'CC-BY-4.0',
	'CC-BY-SA-4.0',
	'CC-BY-NC-4.0',
	'CC-BY-NC-SA-4.0',
	'CC-BY-ND-4.0',
	'CC-BY-NC-ND-4.0',
	'ISC',
	'MIT',
	'Zlib',
]

export const licensesRequiringSource: string[] = [
	'GPL-2.0',
	'GPL-2.0+',
	'GPL-2.0-only',
	'GPL-2.0-or-later',
	'GPL-3.0',
	'GPL-3.0+',
	'GPL-3.0-only',
	'GPL-3.0-or-later',
	'LGPL-2.1',
	'LGPL-2.1+',
	'LGPL-2.1-only',
	'LGPL-2.1-or-later',
	'LGPL-3.0',
	'LGPL-3.0+',
	'LGPL-3.0-only',
	'LGPL-3.0-or-later',
	'AGPL-3.0',
	'AGPL-3.0+',
	'AGPL-3.0-only',
	'AGPL-3.0-or-later',
	'MPL-2.0',
]

export function licenseDoesNotRequireSource(licenseId: string): boolean {
	return licensesNotRequiringSource.includes(licenseId)
}
export function licenseMayRequireSource(licenseId: string): boolean {
	return !licensesNotRequiringSource.includes(licenseId)
}
export function licenseRequiresSource(licenseId: string): boolean {
	return licensesRequiringSource.includes(licenseId)
}

export function notSourceAsDistributed(projectTypes): boolean {
	return projectTypes.includes('mod') || projectTypes.includes('plugin')
}

export function promptSourceRequired(licenseId: string, projectTypes): boolean {
	return licenseRequiresSource(licenseId) && notSourceAsDistributed(projectTypes)
}

export function kebabToTitleCase(input: string): string {
	return input
		.split('-')
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(' ')
}

export function arrayOrNone(arr: string[]): string {
	return arr.length > 0 ? arr.join(', ') : 'None'
}

export function formatProjectTypes(type: string, lower: boolean = false) {
	let value = type
	try {
		value = value
			.replaceAll('mod', 'Mod')
			.replaceAll('resourcepack', 'Resource Pack')
			.replaceAll('datapack', 'Data Pack')
			.replaceAll('plugin', 'Plugin')
			.replaceAll('shader', 'Shaders')
			.replaceAll('minecraft_java_server', 'Server')
			.replaceAll('minecraft_server', 'Server')
	} catch {
		return 'No project type'
	}

	if (lower === true) value = value.toLowerCase()
	return value
}

export function requiresEnvironmentInfo(projectTypes): boolean {
	return projectTypes.includes('mod') || projectTypes.includes('modpack')
}

export function flattenStaticVariables(): Record<string, string> {
	const vars: Record<string, string> = {}

	vars[`RULES`] = `[Modrinth's Content Rules](https://modrinth.com/legal/rules)`
	vars[`R1`] =
		`Per section 1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#prohibited-content)`
	const rule1subs = 12
	for (let n = 1; n <= rule1subs; n++) {
		vars[`R1.${n}`] =
			`Per section 1.${n} of [Modrinth's Content Rules](https://modrinth.com/legal/rules#prohibited-content)`
	}
	vars[`R2`] =
		`Per section 2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#clear-and-honest-function)`
	vars[`R2.1`] =
		`Per section 2.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#general-expectations)`
	const rule2sub1subs = 3
	for (let n = 1; n <= rule2sub1subs; n++) {
		const l = String.fromCharCode(96 + n)
		vars[`R2.1${l}`] =
			`Per section 2.1${l} of [Modrinth's Content Rules](https://modrinth.com/legal/rules#general-expectations)`
	}
	vars[`R2.2`] =
		`Per section 2.2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#accessibility)`
	vars[`R3`] =
		`Per section 3 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#cheats-and-hacks)`
	const rule3subs = 3
	for (let n = 1; n <= rule3subs; n++) {
		vars[`R3.${n}`] =
			`Per section 3.${n} of [Modrinth's Content Rules](https://modrinth.com/legal/rules#cheats-and-hacks)`
	}
	const rule3sub3subs = 6
	for (let n = 1; n <= rule3sub3subs; n++) {
		const l = String.fromCharCode(96 + n)
		vars[`R3.3${l}`] =
			`Per section 3.3${l} of [Modrinth's Content Rules](https://modrinth.com/legal/rules#cheats-and-hacks)`
	}
	vars[`R4`] =
		`Per section 4 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#copyright-and-legality-of-content)`
	vars[`R5`] =
		`Per section 5 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous)`
	const rule5subs = 9
	for (let n = 1; n <= rule5subs; n++) {
		vars[`R5.${n}`] =
			`Per section 5.${n} of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous)`
	}
	vars[`R6`] =
		`Per section 6 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#generative-ai)`
	vars[`R6.1`] =
		`Per section 6.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#disclosure-of-ai-generated-content)`
	const rule6sub1subs = 2
	for (let n = 1; n <= rule6sub1subs; n++) {
		const l = String.fromCharCode(96 + n)
		vars[`R6.1${l}`] =
			`Per section 6.1${l} of [Modrinth's Content Rules](https://modrinth.com/legal/rules#disclosure-of-ai-generated-content)`
	}
	vars[`R6.2`] =
		`Per section 6.2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#prohibited-usage-of-ai)`
	const rule6sub2subs = 4
	for (let n = 1; n <= rule6sub2subs; n++) {
		const l = String.fromCharCode(96 + n)
		vars[`R6.2${l}`] =
			`Per section 6.2${l} of [Modrinth's Content Rules](https://modrinth.com/legal/rules#prohibited-usage-of-ai)`
	}
	vars[`TOS`] = `[Terms of Use](https://modrinth.com/legal/terms)`
	vars[`COPYRIGHT_POLICY`] = `[Copyright Policy](https://modrinth.com/legal/copyright)`
	vars[`SUPPORT`] =
		`please visit the [Modrinth Help Center](https://support.modrinth.com/) and click the blue bubble to contact support.`
	vars[`MODPACK_PERMISSIONS_GUIDE`] =
		`our guide to [Obtaining Modpack Permissions](https://support.modrinth.com/en/articles/8797527-obtaining-modpack-permissions)`
	vars[`MODPACKS_ON_MODRINTH`] =
		`[Modpacks on Modrinth](https://support.modrinth.com/en/articles/8802250-modpacks-on-modrinth)`
	vars[`ADVANCED_MARKDOWN`] =
		`[Markdown Formatting Guide](https://support.modrinth.com/en/articles/8801962-advanced-markdown-formatting)`
	vars[`LICENSING_GUIDE`] =
		`our guide to [Licensing your Mods](https://modrinth.com/news/article/licensing-guide)`
	vars[`NEW_ENVIRONMENTS_LINK`] = `https://modrinth.com/news/article/new-environments`
	vars[`LEARN_MORE_ABOUT_SERVERS_FLINK`] =
		`[learn more about server projects from our news feed](https://modrinth.com/news/article/introducing-server-projects/)`
	vars[`SHARED_INSTANCES_FLINK`] =
		`[Shared Instances](https://modrinth.com/news/article/shared-instances/)`

	return vars
}

export function flattenProjectVariables(
	project: Labrinth.Projects.v2.Project,
): Record<string, string> {
	const vars: Record<string, string> = {}

	vars['PROJECT_ID'] = project.id
	vars['PROJECT_TYPE'] = project.project_type
	vars['PROJECT_SLUG'] = project.slug
	vars['PROJECT_TITLE'] = project.title
	vars['PROJECT_SUMMARY'] = project.description
	vars['PROJECT_STATUS'] = project.status
	vars['PROJECT_REQUESTED_STATUS'] = project.requested_status
	vars['PROJECT_MONETIZATION_STATUS'] = project.monetization_status
	vars['PROJECT_BODY'] = project.body

	vars['PROJECT_ICON_URL'] = project.icon_url || ''
	vars['PROJECT_ISSUES_URL'] = project.issues_url || 'None'
	vars['PROJECT_SOURCE_URL'] = project.source_url || 'None'
	vars['PROJECT_WIKI_URL'] = project.wiki_url || 'None'
	vars['PROJECT_DISCORD_URL'] = project.discord_url || 'None'

	vars['PROJECT_DOWNLOADS'] = project.downloads.toString()
	vars['PROJECT_FOLLOWERS'] = project.followers.toString()
	vars['PROJECT_COLOR'] = project.color?.toString() || ''

	vars['PROJECT_CLIENT_SIDE'] = project.client_side
	vars['PROJECT_SERVER_SIDE'] = project.server_side

	vars['PROJECT_TEAM'] = project.team || 'None'
	vars['PROJECT_THREAD_ID'] = project.thread_id
	vars['PROJECT_ORGANIZATION'] = project.organization

	vars['PROJECT_PUBLISHED'] = project.published
	vars['PROJECT_UPDATED'] = project.updated
	vars['PROJECT_APPROVED'] = project.approved
	vars['PROJECT_QUEUED'] = project.queued

	vars['PROJECT_LICENSE_ID'] = project.license.id
	vars['PROJECT_LICENSE_NAME'] = project.license.name
	vars['PROJECT_LICENSE_URL'] = project.license.url || 'None'

	vars['PROJECT_CATEGORIES'] = arrayOrNone(project.categories)
	vars['PROJECT_ADDITIONAL_CATEGORIES'] = arrayOrNone(project.additional_categories)
	vars['PROJECT_GAME_VERSIONS'] = arrayOrNone(project.game_versions)
	vars['PROJECT_LOADERS'] = arrayOrNone(project.loaders)
	vars['PROJECT_VERSIONS'] = arrayOrNone(project.versions)

	vars['PROJECT_CATEGORIES_COUNT'] = project.categories.length.toString()
	vars['PROJECT_GAME_VERSIONS_COUNT'] = project.game_versions.length.toString()
	vars['PROJECT_LOADERS_COUNT'] = project.loaders.length.toString()
	vars['PROJECT_VERSIONS_COUNT'] = project.versions.length.toString()
	vars['PROJECT_GALLERY_COUNT'] = (project.gallery?.length || 0).toString()
	vars['PROJECT_DONATION_URLS_COUNT'] = project.donation_urls.length.toString()

	project.donation_urls.forEach((donation, index) => {
		vars[`PROJECT_DONATION_${index}_ID`] = donation.id
		vars[`PROJECT_DONATION_${index}_PLATFORM`] = donation.platform
		vars[`PROJECT_DONATION_${index}_URL`] = donation.url
	})

	project.gallery?.forEach((image, index) => {
		vars[`PROJECT_GALLERY_${index}_URL`] = image.url
		vars[`PROJECT_GALLERY_${index}_TITLE`] = image.title || ''
		vars[`PROJECT_GALLERY_${index}_DESCRIPTION`] = image.description || ''
		vars[`PROJECT_GALLERY_${index}_FEATURED`] = image.featured.toString()
	})

	// Navigation related variables
	vars[`PROJECT_PERMANENT_LINK`] = `https://modrinth.com/project/${project.id}`
	vars[`PROJECT_SETTINGS_LINK`] = `https://modrinth.com/project/${project.id}/settings`
	vars[`PROJECT_SETTINGS_FLINK`] = `[Settings](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_ICON_FLINK`] = `[Icon](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_TITLE_FLINK`] = `[Name](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_SLUG_FLINK`] = `[URL](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_SUMMARY_FLINK`] = `[Summary](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_ENVIRONMENT_FLINK`] =
		`[Environment Information](https://modrinth.com/project/${project.id}/settings/environment)` // Depreciated
	vars[`PROJECT_TAGS_LINK`] = `https://modrinth.com/project/${project.id}/settings/tags`
	vars[`PROJECT_TAGS_FLINK`] = `[Tags](https://modrinth.com/project/${project.id}/settings/tags)`
	vars[`PROJECT_DESCRIPTION_LINK`] =
		`https://modrinth.com/project/${project.id}/settings/description`
	vars[`PROJECT_DESCRIPTION_FLINK`] =
		`[Description](https://modrinth.com/project/${project.id}/settings/description)`
	vars[`PROJECT_LICENSE_LINK`] = `https://modrinth.com/project/${project.id}/settings/license`
	vars[`PROJECT_LICENSE_FLINK`] =
		`[License](https://modrinth.com/project/${project.id}/settings/license)`
	vars[`PROJECT_LINKS_LINK`] = `https://modrinth.com/project/${project.id}/settings/links`
	vars[`PROJECT_LINKS_FLINK`] =
		`[External Links](https://modrinth.com/project/${project.id}/settings/links)`
	vars[`PROJECT_GALLERY_LINK`] = `https://modrinth.com/project/${project.id}/gallery`
	vars[`PROJECT_GALLERY_FLINK`] =
		`[Gallery](https://modrinth.com/project/${project.id}/settings/gallery)`
	vars[`PROJECT_VERSIONS_LINK`] = `https://modrinth.com/project/${project.id}/versions`
	vars[`PROJECT_VERSIONS_FLINK`] =
		`[Versions](https://modrinth.com/project/${project.id}/settings/versions)`
	vars[`PROJECT_MODERATION_LINK`] = `https://modrinth.com/project/${project.id}/moderation`
	vars[`PROJECT_MODERATION_FLINK`] =
		`[moderation tab](https://modrinth.com/project/${project.id}/moderation)`
	vars[`PROJECT_SERVER_SETTINGS`] = `https://modrinth.com/project/${project.id}/settings/server`
	vars[`PROJECT_SERVER_SETTINGS_FLINK`] =
		`[Server Settings](https://modrinth.com/project/${project.id}/settings/server)`
	vars[`PROJECT_LANGUAGE_SETTINGS`] = `https://modrinth.com/project/${project.id}/settings/server`
	vars[`PROJECT_LANGUAGE_SETTINGS_FLINK`] =
		`[Language Settings](https://modrinth.com/project/${project.id}/settings/server)`
	vars[`PROJECT_PERMISSIONS_LINK`] =
		`https://modrinth.com/project/${project.id}/settings/permissions`
	vars[`PROJECT_PERMISSIONS_FLINK`] =
		`[Permissions settings](https://modrinth.com/project/${project.id}/settings/permissions)`
	vars[`PROJECT_MONETIZATION_SETTINGS_LINK`] = `https://modrinth.com/project/${project.id}/settings`
	vars[`PROJECT_MONETIZATION_SETTINGS_FLINK`] =
		`[Monetization settings](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_CONTENT_DISCLOSURES_LINK`] =
		`https://modrinth.com/project/${project.id}/settings/disclosures`
	vars[`PROJECT_CONTENT_DISCLOSURES_FLINK`] =
		`[Content Disclosures](https://modrinth.com/project/${project.id}/settings/disclosures)`

	return vars
}

export function flattenProjectV3Variables(
	projectV3: Labrinth.Projects.v3.Project,
): Record<string, string> {
	const vars: Record<string, string> = {}

	const environment = projectV3.environment ?? []
	vars['PROJECT_V3_ENVIRONMENT_COUNT'] = environment.length.toString()
	vars['PROJECT_V3_ALL_ENVIRONMENTS'] = environment.join(', ')

	environment.forEach((env, index) => {
		vars[`PROJECT_V3_ENVIRONMENT_${index}`] = env
	})

	vars['PROJECT_V3_REVIEW_STATUS'] = projectV3.side_types_migration_review_status
	vars['PROJECT_V3_TYPES'] = projectV3.project_types.join(', ')
	vars['PROJECT_TYPE_FORMATTED'] = formatProjectTypes(projectV3.project_types[0])
	vars['PROJECT_TYPE_FORMATTED_LOWER'] = formatProjectTypes(projectV3.project_types[0], true)
	vars['PROJECT_TYPES_FORMATTED'] = formatProjectTypes(projectV3.project_types.join(' / '))
	vars['PROJECT_TYPES_FORMATTED_LOWER'] = formatProjectTypes(
		projectV3.project_types.join(' / '),
		true,
	)

	vars['PROJECT_SITE_URL'] = projectV3.link_urls?.site?.url || 'None'
	vars['PROJECT_STORE_URL'] = projectV3.link_urls?.store?.url || 'None'

	vars['PROJECT_LANGUAGES'] = projectV3.minecraft_server?.languages?.toString() || 'None'
	vars['PROJECT_LANGUAGE_COUNT'] = (projectV3.minecraft_server?.languages?.length || 0).toString()

	return vars
}
