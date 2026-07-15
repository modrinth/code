import { LinkIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import type { NodeBuilder } from '../../types/node'
import { action, group, label, md, mdEscape, stage, toggle } from '../../types/node'

function linkSection(id: string, urlLine: string, ...extra: NodeBuilder[]) {
	return group(id)
		.layout('column')
		.children(
			label(urlLine),
			group().children(
				toggle('misused', 'Misused'),
				toggle('inaccessible', 'Inaccessible'),
				...extra,
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
		.action(
			action()
				.suggestedStatus('flagged')
				.severity('low')
				.message(async (state) => {
					const LINK_NAMES: Record<string, string> = {
						issues: 'Issues',
						source: 'Source',
						wiki: 'Wiki',
						discord: 'Discord',
						site: 'Website',
						store: 'Store',
						...Object.fromEntries(
							Object.values(project.value.link_urls)
								.filter((l) => l.donation)
								.map((l) => [`donation_${l.platform}`, l.platform]),
						),
					}

					const LINK_EXTRAS: Record<string, { misused?: string; inaccessible?: string }> = {
						source: {
							inaccessible: 'checklist/messages/links/note/source_404',
						},
						discord: {
							inaccessible: 'checklist/messages/links/note/discord_inaccessible',
						},
					}

					const sections = Object.entries(state).filter(
						([, s]) => s && typeof s === 'object' && !(s instanceof Set),
					) as [string, Record<string, unknown>][]

					const misused = sections.filter(([, s]) => s.misused === true)
					const inaccessible = sections.filter(([, s]) => s.inaccessible === true)

					if (misused.length === 0 && inaccessible.length === 0) return ''

					let message = await md('checklist/messages/links/header')(state)

					if (misused.length > 0) {
						message += await md('checklist/messages/links/misused_header')(state)
						for (const [id] of misused) {
							message += `- ${LINK_NAMES[id] ?? id}\n`
							const extraPath = LINK_EXTRAS[id]?.misused
							if (extraPath) message += await md(extraPath)(state)
						}
					}

					if (inaccessible.length > 0) {
						message += await md('checklist/messages/links/inaccessible_header')(state)
						for (const [id] of inaccessible) {
							message += `- ${LINK_NAMES[id] ?? id}\n`
							const extraPath = LINK_EXTRAS[id]?.inaccessible
							if (extraPath) message += await md(extraPath)(state)
						}
					}

					return message
				}),
		)
		.children(
			() =>
				project.value.link_urls.issues?.url
					? linkSection(
							'issues',
							`**Issues:** ${project.value.link_urls.issues.url}`,
							toggle('disabled', 'Disabled'),
						)
					: null,

			() =>
				project.value.link_urls.source?.url
					? linkSection(
							'source',
							`**Source:** ${project.value.link_urls.source.url}`,
							toggle('empty', 'Empty Repo'),
						)
					: null,

			() =>
				project.value.link_urls.wiki?.url
					? linkSection(
							'wiki',
							`**Wiki:** ${project.value.link_urls.wiki.url}`,
							toggle('disabled', 'Disabled'),
						)
					: null,

			() =>
				project.value.link_urls.discord?.url
					? linkSection(
							'discord',
							`**Discord:** ${project.value.link_urls.discord.url}`,
							toggle('expiring', 'Expiring'),
						)
					: null,

			() =>
				project.value.link_urls.site?.url
					? linkSection('site', `**Website:** ${project.value.link_urls.site.url}`)
					: null,

			() =>
				project.value.link_urls.store?.url
					? linkSection('store', `**Store:** ${project.value.link_urls.store.url}`)
					: null,

			() =>
				Object.entries(project.value.link_urls)
					.filter(([, l]) => l.donation)
					.map(([, l]) =>
						linkSection(`donation_${l.platform}`, `**${mdEscape(l.platform)}:** ${l.url}`),
					),
		)
}
