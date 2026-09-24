import { defineMessages } from '#ui/composables/i18n'

export const messages = defineMessages({
	header: {
		id: 'update-all-modal.header',
		defaultMessage: 'Update projects',
	},
	project: {
		id: 'update-all-modal.project',
		defaultMessage: 'Project',
	},
	versions: {
		id: 'update-all-modal.versions',
		defaultMessage: 'Current version / New version',
	},
	selectAll: {
		id: 'update-all-modal.select-all',
		defaultMessage: 'Select all projects',
	},
	selectProject: {
		id: 'update-all-modal.select-project',
		defaultMessage: 'Select {project} for update',
	},
	selectVersion: {
		id: 'update-all-modal.select-version',
		defaultMessage: 'New version for {project}',
	},
	viewChangelog: {
		id: 'update-all-modal.view-changelog',
		defaultMessage: 'View changelog for {project}',
	},
	closeChangelog: {
		id: 'update-all-modal.close-changelog',
		defaultMessage: 'Close changelog',
	},
	warning: {
		id: 'update-all-modal.warning',
		defaultMessage:
			'We can’t guarantee updates are safe for your worlds. Review the changelogs and consider a backup.',
	},
	update: {
		id: 'update-all-modal.update',
		defaultMessage: 'Update {count, plural, one {# project} other {# projects}}',
	},
	loading: {
		id: 'update-all-modal.loading',
		defaultMessage: 'Loading updates…',
	},
	empty: {
		id: 'update-all-modal.empty',
		defaultMessage: 'No updates available.',
	},
	noVersion: {
		id: 'update-all-modal.no-version',
		defaultMessage: 'No compatible updates',
	},
	loadingChangelog: {
		id: 'update-all-modal.loading-changelog',
		defaultMessage: 'Loading changelog…',
	},
	noChangelog: {
		id: 'update-all-modal.no-changelog',
		defaultMessage: 'No changelog provided for this version.',
	},
})
