import { defineMessages } from '@modrinth/ui'

export const libraryInstanceMessages = defineMessages({
	install: { id: 'app.library.instance.install', defaultMessage: 'Install' },
	installWithName: {
		id: 'app.library.instance.install-with-name',
		defaultMessage: 'Install {name}',
	},
	installing: { id: 'app.library.instance.installing', defaultMessage: 'Installing...' },
	selectWithName: {
		id: 'app.library.instance.select-with-name',
		defaultMessage: 'Select {name}',
	},
	deselectWithName: {
		id: 'app.library.instance.deselect-with-name',
		defaultMessage: 'Deselect {name}',
	},
	openWithName: {
		id: 'app.library.instance.open-with-name',
		defaultMessage: 'Open {name}',
	},
	hideFromLibrary: {
		id: 'app.library.instance.action.hide-from-library',
		defaultMessage: 'Hide from library',
	},
	showInLibrary: {
		id: 'app.library.instance.action.show-in-library',
		defaultMessage: 'Show in library',
	},
	sharedBy: {
		id: 'app.library.instance.shared-by',
		defaultMessage: 'Shared by {name}',
	},
	sharedByOwner: {
		id: 'app.library.instance.shared-by-owner',
		defaultMessage: 'Shared by <owner>{name}</owner>',
	},
	sharedByYou: {
		id: 'app.library.instance.shared-by-you',
		defaultMessage: 'Shared by you',
	},
})
