import { LinkIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed } from 'vue'

import { group, md, stage, toggle } from '../../types/node'
import type { ChildEntry, GroupNodeBuilder } from '../../types/node'
import { promptSourceRequired } from '../..'

export default function () {
	const { projectV3: project } = injectProjectPageContext()
	const linkNames: Record<string, string> = {}

	type LinkSectionBuilder = GroupNodeBuilder & {
		children(...extras: ChildEntry[]): LinkSectionBuilder
		label(badge: Ref<boolean>): LinkSectionBuilder
	}

	function linkSection(id: string, name: string): LinkSectionBuilder {
		linkNames[id] = name
		let showBadge: Ref<boolean> | undefined
		const url = computed(() => project.value.link_urls[id]?.url)

		const inner = group().children(
			toggle('misused', 'Misused'),
			toggle('inaccessible', 'Inaccessible'),
		)

		const outer = group(id)
			.layout('column')
			.shown(computed(() => !!url.value))
			.children(
				() => (
					<div class="markdown-body w-full">
						{showBadge?.value && (
							<>
								<strong>[📜]</strong>{' '}
							</>
						)}
						<strong>{name}:</strong>{' '}
						<a href={url.value} target="_blank" class="underline">
							{url.value}
						</a>
					</div>
				),
				inner,
			)

		const builder = Object.assign(outer, {
			children(...extras: ChildEntry[]) {
				inner.children(...extras)
				return builder
			},
			label(badge: Ref<boolean>) {
				showBadge = badge
				return builder
			},
		}) as LinkSectionBuilder

		return builder
	}

	const showWarning = computed(() =>
		promptSourceRequired(project.value.license.id, project.value.project_types),
	)

	const misusedKeys = new Set(['misused', 'empty'])

	return stage('links', 'Links')
		.hint("Are the project's links accurate and accessible?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
		)
		.icon(LinkIcon)
		.navigate('/settings/links')
		.shown(computed(() => Object.keys(project.value.link_urls).length > 0))
		.suggestedStatus('flagged')
		.severity('low')
		.rawMessage(async (state) => {
			const sections = Object.entries(state).filter(
				([, s]) => s && typeof s === 'object' && !(s instanceof Set),
			) as [string, Record<string, unknown>][]

			const misused = sections.flatMap(([id, s]) =>
				Object.keys(s)
					.filter((t) => s[t] === true && misusedKeys.has(t))
					.map((t) => [id, t] as const),
			)
			const inaccessible = sections.flatMap(([id, s]) =>
				Object.keys(s)
					.filter((t) => s[t] === true && !misusedKeys.has(t))
					.map((t) => [id, t] as const),
			)

			if (!misused.length && !inaccessible.length) return ''

			const linkLine = (id: string) =>
				`- ${linkNames[id] ?? id}: \`${project.value.link_urls[id]?.url}\`\n`

			const renderGroup = async (pairs: readonly (readonly [string, string])[]) => {
				const byId = new Map<string, string[]>()
				for (const [id, t] of pairs) {
					if (!byId.has(id)) byId.set(id, [])
					byId.get(id)!.push(t)
				}
				let out = ''
				for (const [id, toggles] of byId) {
					out += linkLine(id)
					for (const t of toggles) {
						let note = (await md(`checklist/messages/links/${id}/${t}`)(state)).trim()
						if (!note && (t === 'inaccessible' || t === 'misused')) {
							note = (
								await md(`checklist/messages/links/${t}`, () => ({
									LINK_NAME: linkNames[id] ?? id,
									LINK_URL: project.value.link_urls[id]?.url ?? '',
								}))(state)
							).trim()
						}
						if (note) out += `    - ${note}\n`
					}
				}
				return out
			}

			let message = await md('checklist/messages/links/header')(state)

			if (misused.length) {
				message += `</br>\n\n**These links appear to be misused or not related to your project:**\n\n`
				message += await renderGroup(misused)
			}

			if (inaccessible.length) {
				message += `</br>\n\n**These links appear to be inaccessible:**\n\n`
				message += await renderGroup(inaccessible)
			}

			return message
		})
		.children(
			linkSection('issues', 'Issue tracker').children(toggle('disabled', 'Disabled')),

			linkSection('source', 'Source code')
				.label(showWarning)
				.children(toggle('empty', 'Empty Repo')),

			linkSection('wiki', 'Wiki page').children(toggle('disabled', 'Disabled')),

			linkSection('discord', 'Discord invite').children(toggle('expiring', 'Expiring')),

			linkSection('site', 'Website'),
			linkSection('store', 'Store'),

			group('donations')
				.layout('column')
				.children(
					linkSection('patreon', 'Patreon'),
					linkSection('bmac', 'Buy Me A Coffee'),
					linkSection('paypal', 'PayPal'),
					linkSection('github', 'GitHub Sponsors'),
					linkSection('ko-fi', 'Ko-fi'),
					linkSection('other', 'Other'),
				),
		)
}
