import { defineMessages } from '@modrinth/ui'

export const messages = defineMessages({
	title: { id: 'project.settings.links.title', defaultMessage: 'Links' },
	linksHelp: {
		id: 'project.settings.links.external-resources-help',
		defaultMessage:
			'Link to external resources that give users more information about your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}.',
	},
	issues: { id: 'project.settings.links.issues', defaultMessage: 'Issue tracker' },
	issuesDescription: {
		id: 'project.settings.links.issues-description',
		defaultMessage: 'A place for users to report bugs, issues, and concerns about your project.',
	},
	source: { id: 'project.settings.links.source', defaultMessage: 'Source code' },
	sourceDescription: {
		id: 'project.settings.links.source-description',
		defaultMessage: 'A page/repository containing the source code for your project',
	},
	wiki: { id: 'project.settings.links.wiki', defaultMessage: 'Wiki page' },
	wikiDescription: {
		id: 'project.settings.links.wiki-description',
		defaultMessage: 'A page containing information, documentation, and help for the project.',
	},
	discord: { id: 'project.settings.links.discord', defaultMessage: 'Discord invite' },
	discordDescription: {
		id: 'project.settings.links.discord-description',
		defaultMessage: 'An invitation link to your Discord server.',
	},
	site: { id: 'project.settings.links.site', defaultMessage: 'Website' },
	siteDescription: {
		id: 'project.settings.links.site-description',
		defaultMessage: "Your server's official website.",
	},
	store: { id: 'project.settings.links.store', defaultMessage: 'Store' },
	storeDescription: {
		id: 'project.settings.links.store-description',
		defaultMessage: "A link to your server's store or shop.",
	},
	linkType: { id: 'project.settings.links.link-type', defaultMessage: 'Link type' },
	donationPlatform: { id: 'project.settings.links.donation-platform', defaultMessage: 'Platform' },
	donationLink: { id: 'project.settings.links.donation-link', defaultMessage: 'Link' },
	donations: { id: 'project.settings.links.donations', defaultMessage: 'Donation links' },
	addLink: { id: 'project.settings.links.add-link', defaultMessage: 'Add link' },
	removeDonation: {
		id: 'project.settings.links.remove-donation-link',
		defaultMessage: 'Remove donation link',
	},
	donationsDescription: {
		id: 'project.settings.links.donations-description',
		defaultMessage: 'Add donation links for users to support you directly.',
	},
	donationsEmptyState: {
		id: 'project.settings.links.donations-empty-state',
		defaultMessage: 'Donation links let users support you directly.',
	},
	donationsHelp: {
		id: 'project.settings.links.donation-platforms-help',
		defaultMessage: 'Link to platforms where you accept donations for your work on this content.',
	},
	urlPlaceholder: {
		id: 'project.settings.links.url-placeholder',
		defaultMessage: 'Enter a valid URL',
	},
	donationUsernamePlaceholder: {
		id: 'project.settings.links.donation-username-placeholder',
		defaultMessage: 'Enter your {platform} username',
	},
	visitLink: {
		id: 'project.settings.links.visit-link',
		defaultMessage: 'Visit {url}',
	},
	donationUrl: {
		id: 'project.settings.links.donation-url',
		defaultMessage: 'URL',
	},
	donationUsername: {
		id: 'project.settings.links.donation-username',
		defaultMessage: 'Username',
	},
	updatedTitle: { id: 'project.settings.links.updated-title', defaultMessage: 'Links updated' },
	updated: {
		id: 'project.settings.links.updated',
		defaultMessage: 'Your links have been updated.',
	},
	serverDiscord: { id: 'project.settings.links.server-discord', defaultMessage: 'Discord' },
	serverWikiDescription: {
		id: 'project.settings.links.server-wiki-description',
		defaultMessage: 'A page containing information, documentation, and help for the server.',
	},
	serverUpdated: {
		id: 'project.settings.links.server-updated',
		defaultMessage: 'Your server links have been updated.',
	},
})
