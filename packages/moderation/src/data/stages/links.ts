import { LinkIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, group, label, mdEscape, stage, toggle } from '../../types/node'

function linkSection(id: string, urlLine: string) {
	return group(id)
		.layout('column')
		.children(
			label(urlLine),
			group().children(
				toggle('misused', 'Misused').action(
					action().suggestedStatus('flagged').severity('low').message(),
				),
				toggle('inaccessible', 'Inaccessible').action(
					action().suggestedStatus('flagged').severity('medium').message(),
				),
			),
		)
}

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('links', 'Links')
		.hint("Are the project's links accurate and accessible?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
		)
		.icon(LinkIcon)
		.navigate('/settings/links')
		.shown(computed(() => Object.keys(project.value.link_urls).length > 0))
		.children(
			() =>
				project.value.link_urls.issues?.url
					? linkSection('issues', `**Issues:** ${mdEscape(project.value.link_urls.issues.url)}`)
					: null,

			() =>
				project.value.link_urls.source?.url
					? linkSection('source', `**Source:** ${mdEscape(project.value.link_urls.source.url)}`)
					: null,

			() =>
				project.value.link_urls.wiki?.url
					? linkSection('wiki', `**Wiki:** ${mdEscape(project.value.link_urls.wiki.url)}`)
					: null,

			() =>
				project.value.link_urls.discord?.url
					? linkSection('discord', `**Discord:** ${mdEscape(project.value.link_urls.discord.url)}`)
					: null,

			() =>
				project.value.link_urls.site?.url
					? linkSection('site', `**Website:** ${mdEscape(project.value.link_urls.site.url)}`)
					: null,

			() =>
				project.value.link_urls.store?.url
					? linkSection('store', `**Store:** ${mdEscape(project.value.link_urls.store.url)}`)
					: null,

			() =>
				Object.values(project.value.link_urls).some((l) => l.donation)
					? group('donations')
							.layout('column')
							.children(
								label(
									Object.values(project.value.link_urls)
										.filter((l) => l.donation)
										.map((l) => `**${mdEscape(l.platform)}:** ${mdEscape(l.url)}`)
										.join(' \\\n'),
								),
								group().children(
									toggle('misused', 'Misused').action(
										action().suggestedStatus('flagged').severity('low').message(),
									),
									toggle('inaccessible', 'Inaccessible').action(
										action().suggestedStatus('flagged').severity('medium').message(),
									),
								),
							)
					: null,
		)
}
