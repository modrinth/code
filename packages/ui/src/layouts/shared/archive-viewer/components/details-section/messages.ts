import { defineMessages } from '#ui/composables/i18n'

export const sectionMessages = defineMessages({
	copyPath: { id: 'archive-viewer.details.copy-path', defaultMessage: 'Copy path' },
	pathCopied: { id: 'archive-viewer.details.path-copied', defaultMessage: 'Path copied' },
	contentsCounts: {
		id: 'archive-viewer.details.contents-counts',
		defaultMessage: 'Contents',
	},
	filesCount: {
		id: 'archive-viewer.details.files-count',
		defaultMessage: '{count, plural, one {# file} other {# files}}',
	},
	foldersCount: {
		id: 'archive-viewer.details.folders-count',
		defaultMessage: '{count, plural, one {# folder} other {# folders}}',
	},
	classFilesCount: {
		id: 'archive-viewer.details.class-files-count',
		defaultMessage: '{count, plural, one {# class file} other {# class files}}',
	},
	nestedArchivesCount: {
		id: 'archive-viewer.details.nested-archives-count',
		defaultMessage: '{count, plural, one {# nested archive} other {# nested archives}}',
	},
	statusReading: {
		id: 'archive-viewer.details.status-reading',
		defaultMessage: 'Reading nested archives...',
	},
	statusPending: { id: 'archive-viewer.details.status-pending', defaultMessage: 'Not read yet' },
	statusRead: { id: 'archive-viewer.details.status-read', defaultMessage: 'Fully read' },
	copyValue: { id: 'archive-viewer.details.copy-value', defaultMessage: 'Copy {label}' },
	valueCopied: { id: 'archive-viewer.details.value-copied', defaultMessage: '{label} copied' },
	showDetails: { id: 'archive-viewer.details.show-details', defaultMessage: 'Show details' },
	hideDetails: { id: 'archive-viewer.details.hide-details', defaultMessage: 'Hide details' },
})
