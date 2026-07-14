import { LinkIcon } from '@modrinth/assets'

import { action, toggle, group, label, md, mdEscape, stage, stageFn } from '../../types/node'

function linkSection(id: string, urlLine: string) {
	return group(id)
		.layout('column')
		.children(
			label(urlLine),
			group().children(
				toggle('misused', 'Misused')
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('low')
							.message(md(`checklist/messages/links/${id}/misused`)),
					),
				toggle('inaccessible', 'Inaccessible')
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('medium')
							.message(md(`checklist/messages/links/${id}/inaccessible`)),
					),
			),
		)
}

export default stageFn((project) => stage('links', 'Links')
	.hint("Are the project's links accurate and accessible?")
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177')
	.icon(LinkIcon)
	.navigate('/settings/links')
	.shown(Object.keys(project.link_urls).length > 0)
	.children(
		linkSection('issues', `**Issues:** ${mdEscape(project.link_urls.issues?.url ?? '')}`)
			.shown(!!project.link_urls.issues?.url),

		linkSection('source', `**Source:** ${mdEscape(project.link_urls.source?.url ?? '')}`)
			.shown(!!project.link_urls.source?.url),

		linkSection('wiki', `**Wiki:** ${mdEscape(project.link_urls.wiki?.url ?? '')}`)
			.shown(!!project.link_urls.wiki?.url),

		linkSection('discord', `**Discord:** ${mdEscape(project.link_urls.discord?.url ?? '')}`)
			.shown(!!project.link_urls.discord?.url),

		linkSection('site', `**Website:** ${mdEscape(project.link_urls.site?.url ?? '')}`)
			.shown(!!project.link_urls.site?.url),

		linkSection('store', `**Store:** ${mdEscape(project.link_urls.store?.url ?? '')}`)
			.shown(!!project.link_urls.store?.url),

		group('donations')
			.layout('column')
			.shown(Object.values(project.link_urls).some((l) => l.donation))
			.children(
				label(
					Object.values(project.link_urls)
						.filter((l) => l.donation)
						.map((l) => `**${mdEscape(l.platform)}:** ${mdEscape(l.url)}`)
						.join(' \\\n'),
				),
				group().children(
					toggle('misused', 'Misused')
						.action(
							action()
								.suggestedStatus('flagged')
								.severity('low')
								.message(md('checklist/messages/links/donations/misused')),
						),
					toggle('inaccessible', 'Inaccessible')
						.action(
							action()
								.suggestedStatus('flagged')
								.severity('medium')
								.message(md('checklist/messages/links/donations/inaccessible')),
						),
				),
			),
	),
)
