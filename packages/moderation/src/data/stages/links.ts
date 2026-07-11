import { LinkIcon } from '@modrinth/assets'

import type { ContentFn } from '../../types/node'
import { action, button, group, label, mdEscape, md, stage } from '../../types/node'

function linkSection(id: string, urlLine: ContentFn, baseWeight: number) {
	return group(id)
		.layout('column')
		.children(
			label(urlLine),
			group().children(
				button('misused', 'Misused')
					.action(
						action()
							.weight(baseWeight)
							.suggestedStatus('flagged')
							.severity('low')
							.message(md(`checklist/messages/links/${id}/misused`)),
					),
				button('inaccessible', 'Inaccessible')
					.action(
						action()
							.weight(baseWeight + 1)
							.suggestedStatus('flagged')
							.severity('medium')
							.message(md(`checklist/messages/links/${id}/inaccessible`)),
					),
			),
		)
}

export default stage(
	'links',
	'Links',
	"Are the project's links accurate and accessible?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
)
	.icon(LinkIcon)
	.navigate('/settings/links')
	.shown(({ project }) => Object.keys(project.link_urls).length > 0)
	.children(
		linkSection(
			'issues',
			({ project }) => `**Issues:** ${mdEscape(project.link_urls.issues?.url ?? '')}`,
			500,
		).shown(({ project }) => !!project.link_urls.issues?.url),

		linkSection(
			'source',
			({ project }) => `**Source:** ${mdEscape(project.link_urls.source?.url ?? '')}`,
			510,
		).shown(({ project }) => !!project.link_urls.source?.url),

		linkSection(
			'wiki',
			({ project }) => `**Wiki:** ${mdEscape(project.link_urls.wiki?.url ?? '')}`,
			520,
		).shown(({ project }) => !!project.link_urls.wiki?.url),

		linkSection(
			'discord',
			({ project }) => `**Discord:** ${mdEscape(project.link_urls.discord?.url ?? '')}`,
			530,
		).shown(({ project }) => !!project.link_urls.discord?.url),

		linkSection(
			'site',
			({ project }) => `**Website:** ${mdEscape(project.link_urls.site?.url ?? '')}`,
			540,
		).shown(({ project }) => !!project.link_urls.site?.url),

		linkSection(
			'store',
			({ project }) => `**Store:** ${mdEscape(project.link_urls.store?.url ?? '')}`,
			550,
		).shown(({ project }) => !!project.link_urls.store?.url),

		group('donations')
			.layout('column')
			.shown(({ project }) => Object.values(project.link_urls).some((l) => l.donation))
			.children(
				label(({ project }) =>
					Object.values(project.link_urls)
						.filter((l) => l.donation)
						.map((l) => `**${mdEscape(l.platform)}:** ${mdEscape(l.url)}`)
						.join(' \\\n'),
				),
				group().children(
					button('misused', 'Misused')
						.action(
							action()
								.weight(560)
								.suggestedStatus('flagged')
								.severity('low')
								.message(md('checklist/messages/links/donations/misused')),
						),
					button('inaccessible', 'Inaccessible')
						.action(
							action()
								.weight(561)
								.suggestedStatus('flagged')
								.severity('medium')
								.message(md('checklist/messages/links/donations/inaccessible')),
						),
				),
			),
	)
