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
	projectList: {
		id: 'update-all-modal.project-list',
		defaultMessage: 'Project updates',
	},
	versions: {
		id: 'update-all-modal.versions',
		defaultMessage: 'Current version / New version',
	},
	currentVersion: {
		id: 'update-all-modal.current-version',
		defaultMessage: 'Current version',
	},
	newVersion: {
		id: 'update-all-modal.new-version',
		defaultMessage: 'New version',
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
	changelogFor: {
		id: 'update-all-modal.changelog-for',
		defaultMessage: 'Changelog for {project}',
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
	results: {
		id: 'update-all-modal.results',
		defaultMessage: '{count, plural, one {# project ready to update} other {# projects ready to update}}',
	},
	noCompatibleUpdates: {
		id: 'update-all-modal.no-compatible-updates',
		defaultMessage: 'No compatible updates available.',
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
	changelogLoaded: {
		id: 'update-all-modal.changelog-loaded',
		defaultMessage: 'Changelog loaded.',
	},
	noChangelog: {
		id: 'update-all-modal.no-changelog',
		defaultMessage: 'No changelog provided for this version.',
	},
})
