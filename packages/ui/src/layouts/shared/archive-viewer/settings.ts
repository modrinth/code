import {
	defineSetting,
	type EnumSettingDefinition,
	type SettingDefinitionBase,
	type SettingGroup,
	type SettingsStore,
	useLocalSettings,
} from '#ui/components/settings'
import { defineMessages, type MessageDescriptor } from '#ui/composables/i18n'
import { AVAILABLE_DECOMPILER_TYPES, type DecompilerTypes } from '#ui/helpers/toolbox/decompile'
import { HASH_TYPES, type HashType } from '#ui/helpers/toolbox/file'
import { createContext } from '#ui/providers/create-context'

import { decompilerMessages, hashTypeMessages } from './messages'

/** When a detail shows: always, only once its item's details are pulled out, or never. */
export type DetailVisibility = 'always' | 'details' | 'never'

const messages = defineMessages({
	always: { id: 'archive-viewer.settings.visibility.always', defaultMessage: 'Always' },
	details: { id: 'archive-viewer.settings.visibility.details', defaultMessage: 'In details' },
	never: { id: 'archive-viewer.settings.visibility.never', defaultMessage: 'Never' },

	sections: { id: 'archive-viewer.settings.sections', defaultMessage: 'Details stack' },
	sectionsDescription: {
		id: 'archive-viewer.settings.sections-description',
		defaultMessage: 'Which items the details panel describes, from the outside in.',
	},
	source: { id: 'archive-viewer.settings.section.source', defaultMessage: 'Project and version' },
	sourceDescription: {
		id: 'archive-viewer.settings.section.source-description',
		defaultMessage: 'Where the archive was published, when the viewer knows.',
	},
	root: { id: 'archive-viewer.settings.section.root', defaultMessage: 'Opened archive' },
	rootDescription: {
		id: 'archive-viewer.settings.section.root-description',
		defaultMessage: 'The archive itself: where it came from, its size, hashes and contents.',
	},
	nested: { id: 'archive-viewer.settings.section.nested', defaultMessage: 'Nested archives' },
	nestedDescription: {
		id: 'archive-viewer.settings.section.nested-description',
		defaultMessage: 'Archives stored within the opened one, down to the one holding the open file.',
	},
	entry: { id: 'archive-viewer.settings.section.entry', defaultMessage: 'Open file or folder' },
	entryDescription: {
		id: 'archive-viewer.settings.section.entry-description',
		defaultMessage: 'The file or folder open in the active tab.',
	},

	information: { id: 'archive-viewer.settings.information', defaultMessage: 'Information' },
	informationDescription: {
		id: 'archive-viewer.settings.information-description',
		defaultMessage:
			'When each kind of detail shows: always, only with details pulled out, or never.',
	},
	detectedType: { id: 'archive-viewer.settings.detected-type', defaultMessage: 'Detected type' },
	detectedTypeDescription: {
		id: 'archive-viewer.settings.detected-type-description',
		defaultMessage: "A file's type, as detected from its contents.",
	},
	detectedLanguage: {
		id: 'archive-viewer.settings.detected-language',
		defaultMessage: 'Detected language',
	},
	detectedLanguageDescription: {
		id: 'archive-viewer.settings.detected-language-description',
		defaultMessage: 'The language a text file is highlighted as.',
	},
	modrinthLookup: {
		id: 'archive-viewer.settings.modrinth-lookup',
		defaultMessage: 'On Modrinth',
	},
	modrinthLookupDescription: {
		id: 'archive-viewer.settings.modrinth-lookup-description',
		defaultMessage:
			'Whether nested archives (and archives opened from other links) are already published on Modrinth, found by their hash.',
	},
	size: { id: 'archive-viewer.settings.size', defaultMessage: 'Size' },
	sizeDescription: {
		id: 'archive-viewer.settings.size-description',
		defaultMessage: 'Sizes as stored and in full.',
	},
	contentCounts: {
		id: 'archive-viewer.settings.content-counts',
		defaultMessage: 'Contents counts',
	},
	contentCountsDescription: {
		id: 'archive-viewer.settings.content-counts-description',
		defaultMessage: 'How many files, folders, class files and nested archives an archive holds.',
	},

	files: { id: 'archive-viewer.settings.files', defaultMessage: 'Class files' },
	filesDescription: {
		id: 'archive-viewer.settings.files-description',
		defaultMessage: 'How compiled Java classes are turned back into source.',
	},
	decompiler: { id: 'archive-viewer.settings.decompiler', defaultMessage: 'Default decompiler' },
	decompilerDescription: {
		id: 'archive-viewer.settings.decompiler-description',
		defaultMessage:
			'The decompiler class files open with. Each tab can switch to another from its navigation.',
	},
	hashes: { id: 'archive-viewer.settings.hashes', defaultMessage: 'Hashes' },
	hashesDescription: {
		id: 'archive-viewer.settings.hashes-description',
		defaultMessage:
			'Hashes are only computed once shown. The SHA family is computed by the browser, Murmur2 is the fingerprint CurseForge identifies files by, and CRC-32 is the checksum archives store for their files.',
	},
})

const VISIBILITY_ENTRIES = [
	{ value: 'always', label: messages.always },
	{ value: 'details', label: messages.details },
	{ value: 'never', label: messages.never },
] as const

function visibilitySetting(
	id: string,
	title: MessageDescriptor,
	description: MessageDescriptor | undefined,
	defaultValue: DetailVisibility,
): EnumSettingDefinition<DetailVisibility> {
	return defineSetting.enum<DetailVisibility>({
		id,
		title,
		description,
		default: defaultValue,
		display: 'chips',
		entries: VISIBILITY_ENTRIES,
	})
}

const DEFAULT_HASH_VISIBILITY: Record<HashType, DetailVisibility> = {
	sha1: 'details',
	sha256: 'never',
	sha384: 'never',
	sha512: 'details',
	murmur2: 'details',
	crc32: 'details',
}

export const archiveSettings = {
	sections: {
		source: defineSetting.toggle({
			id: 'section-source',
			title: messages.source,
			description: messages.sourceDescription,
			default: true,
		}),
		root: defineSetting.toggle({
			id: 'section-root',
			title: messages.root,
			description: messages.rootDescription,
			default: true,
		}),
		nested: defineSetting.toggle({
			id: 'section-nested',
			title: messages.nested,
			description: messages.nestedDescription,
			default: true,
		}),
		entry: defineSetting.toggle({
			id: 'section-entry',
			title: messages.entry,
			description: messages.entryDescription,
			default: true,
		}),
	},
	information: {
		detectedType: visibilitySetting(
			'detected-type',
			messages.detectedType,
			messages.detectedTypeDescription,
			'always',
		),
		detectedLanguage: visibilitySetting(
			'detected-language',
			messages.detectedLanguage,
			messages.detectedLanguageDescription,
			'details',
		),
		size: visibilitySetting('size', messages.size, messages.sizeDescription, 'always'),
		modrinthLookup: visibilitySetting(
			'modrinth-lookup',
			messages.modrinthLookup,
			messages.modrinthLookupDescription,
			'always',
		),
		contentCounts: visibilitySetting(
			'content-counts',
			messages.contentCounts,
			messages.contentCountsDescription,
			'details',
		),
	},
	files: {
		decompiler: defineSetting.enum<DecompilerTypes>({
			id: 'decompiler',
			title: messages.decompiler,
			description: messages.decompilerDescription,
			default: 'vineflower',
			display: 'chips',
			entries: AVAILABLE_DECOMPILER_TYPES.map((type) => ({
				value: type,
				label: decompilerMessages[type],
			})),
		}),
	},
	hashes: Object.fromEntries(
		HASH_TYPES.map((type) => [
			type,
			visibilitySetting(
				`hash-${type}`,
				hashTypeMessages[type],
				undefined,
				DEFAULT_HASH_VISIBILITY[type],
			),
		]),
	) as Record<HashType, EnumSettingDefinition<DetailVisibility>>,
}

/** The archive viewer's own settings, by heading. */
export const archiveSettingGroups: SettingGroup[] = [
	{
		id: 'sections',
		title: messages.sections,
		description: messages.sectionsDescription,
		settings: Object.values(archiveSettings.sections),
	},
	{
		id: 'information',
		title: messages.information,
		description: messages.informationDescription,
		settings: Object.values(archiveSettings.information),
	},
	{
		id: 'files',
		title: messages.files,
		description: messages.filesDescription,
		settings: Object.values(archiveSettings.files),
	},
	{
		id: 'hashes',
		title: messages.hashes,
		description: messages.hashesDescription,
		settings: HASH_TYPES.map((type) => archiveSettings.hashes[type]),
	},
]

/** The workbench panel holding the archive viewer's settings. */
export const SETTINGS_PANEL_ID = 'settings'

/** Where the archive viewer's settings are kept; hosts can provide their own store. */
export const [injectArchiveSettings, provideArchiveSettings] = createContext<SettingsStore>(
	'ArchiveViewerLayout',
	'archiveSettings',
)

/** The archive viewer's settings, kept in local storage, for hosts without settings of their own. */
export function useStoredArchiveSettings() {
	return useLocalSettings(
		archiveSettingGroups.flatMap((group) => group.settings),
		{ prefix: 'archive-viewer-setting:' },
	)
}

/** The settings as the details panel reads them, falling back to the defaults without a store. */
export function useArchiveDetailSettings() {
	const store = injectArchiveSettings(null)

	function get<T>(definition: SettingDefinitionBase<T>): T {
		return store ? store.get(definition) : definition.default
	}

	return {
		section: (key: keyof typeof archiveSettings.sections) => get(archiveSettings.sections[key]),
		visibility: (key: keyof typeof archiveSettings.information) =>
			get(archiveSettings.information[key]),
		hashVisibility: (type: HashType) => get(archiveSettings.hashes[type]),
	}
}
