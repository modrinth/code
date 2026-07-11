import { LinkIcon } from '@modrinth/assets'

import type { ContentFn } from '../../types/node'
import { button, group, mdEscape, mdMsg, prose, stage } from '../../types/node'

function linkSection(id: string, urlLine: ContentFn, baseWeight: number) {
	return group(id)
		.column()
		.children(
			prose(urlLine),
			group().children(
				button('misused', 'Misused')
					.weight(baseWeight)
					.suggestedStatus('flagged')
					.severity('low')
					.message(mdMsg(`links/${id}/misused`)),
				button('inaccessible', 'Inaccessible')
					.weight(baseWeight + 1)
					.suggestedStatus('flagged')
					.severity('medium')
					.message(mdMsg(`links/${id}/inaccessible`)),
			),
		)
}

export default stage(
	'links',
	'Links',
	"Are the project's links accurate and accessible?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	{
		icon: LinkIcon,
		navigate: '/settings/links',
		shown: (_project, projectV3) =>
			Boolean(projectV3 && Object.keys(projectV3.link_urls).length > 0),
	},
	[
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
			.column()
			.shown(({ project }) => Object.values(project.link_urls).some((l) => l.donation))
			.children(
				prose(({ project }) =>
					Object.values(project.link_urls)
						.filter((l) => l.donation)
						.map((l) => `**${mdEscape(l.platform)}:** ${mdEscape(l.url)}`)
						.join(' \\\n'),
				),
				group().children(
					button('misused', 'Misused')
						.weight(560)
						.suggestedStatus('flagged')
						.severity('low')
						.message(mdMsg('links/donations/misused')),
					button('inaccessible', 'Inaccessible')
						.weight(561)
						.suggestedStatus('flagged')
						.severity('medium')
						.message(mdMsg('links/donations/inaccessible')),
				),
			),
	],
)
