import { ListBulletedIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import {
	action,
	group,
	markdown,
	md,
	type MessageFn,
	option,
	stage,
	toggle,
} from '../../types/node'

//TODO: coolbot have fun :3
export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const prohibitedContentHeaderMsg: MessageFn = md(
		'checklist/messages/otherRules/prohibitedContentHeader',
	)
	const prohibitedContentMsgs: Record<string, MessageFn> = {
		objectionable: md('checklist/messages/otherRules/prohibitedContent/objectionable'),
		discriminatory: md('checklist/messages/otherRules/prohibitedContent/discriminatory'),
		ipInfringement: md('checklist/messages/otherRules/prohibitedContent/ipInfringement'),
		legalRights: md('checklist/messages/otherRules/prohibitedContent/legalRights'),
		illegalActivity: md('checklist/messages/otherRules/prohibitedContent/illegalActivity'),
		harmful: md('checklist/messages/otherRules/prohibitedContent/harmful'),
		misleading: md('checklist/messages/otherRules/prohibitedContent/misleading'),
		impersonation: md('checklist/messages/otherRules/prohibitedContent/impersonation'),
		falseEndorsement: md('checklist/messages/otherRules/prohibitedContent/falseEndorsement'),
		profanity: md('checklist/messages/otherRules/prohibitedContent/profanity'),
		undisclosedUpload: md('checklist/messages/otherRules/prohibitedContent/undisclosedUpload'),
		mojangBypass: md('checklist/messages/otherRules/prohibitedContent/mojangBypass'),
	}
	const serverSideOptInHeaderMsg: MessageFn = md(
		'checklist/messages/otherRules/serverSideOptInHeader',
	)
	const serverSideOptInMsgs: Record<string, MessageFn> = {
		xRay: md('checklist/messages/otherRules/serverSideOptIn/xRay'),
		aimBot: md('checklist/messages/otherRules/serverSideOptIn/aimBot'),
		movement: md('checklist/messages/otherRules/serverSideOptIn/movement'),
		pvp: md('checklist/messages/otherRules/serverSideOptIn/pvp'),
		hidingMods: md('checklist/messages/otherRules/serverSideOptIn/hidingMods'),
		itemDuplication: md('checklist/messages/otherRules/serverSideOptIn/itemDuplication'),
	}

	return stage('otherRules', 'Rule Following')
		.hint('Does this project violate the rules?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
		)
		.icon(ListBulletedIcon)
		.navigate('/moderation')
		.children(
			group().children(
				toggle('paidAccessServer', 'Paid access server')
					.shown(computed(() => !!project.value.minecraft_server))
					.action(action().suggestedStatus('rejected').severity('critical').message()),

				// TODO: chyz, the lists built by these message have empty line gaps.
				toggle('prohibitedContent', 'Prohibited Content')
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('critical')
							.message(async (state) => {
								const header = (await prohibitedContentHeaderMsg(state)).trimEnd()
								const selected = state.options
								if (!(selected instanceof Set) || selected.size === 0) return header
								const items = await Promise.all(
									[...selected].map(
										(id) => prohibitedContentMsgs[id]?.(state) ?? Promise.resolve(''),
									),
								)
								return `${header}\n${items.join('\n')}`
							}),
					)
					.children(
						group()
							.multiSelect('options')
							.title('Which Prohibited Content rules does this project violate?')
							.children(
								option('objectionable', 'Objectionable'),
								option('discriminatory', 'Discriminatory or Explicit'),
								option('ipInfringement', 'IP Infringement'),
								option('legalRights', 'Rights Violation'),
								option('illegalActivity', 'Illegal Activity'),
								option('harmful', 'Harmful or Deceptive'),
								option('misleading', 'Misleading claims'),
								option('impersonation', 'Impersonation'),
								option('falseEndorsement', 'False Endorsement'),
								option('profanity', 'Profanity'),
								option('undisclosedUpload', 'Undisclosed Data Upload'),
								option('mojangBypass', 'Mojang Bypass'),
							),
					),

				toggle('cheatOrHackAdvertising', 'Hacks').action(
					action().suggestedStatus('rejected').severity('critical').message(),
				),

				toggle('serverSideOptOut', 'Opt-out').action(
					action().suggestedStatus('flagged').severity('high').message(),
				),

				toggle('serverSideOptIn', 'Opt-in')
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('high')
							.message(async (state) => {
								const header = (await serverSideOptInHeaderMsg(state)).trimEnd()
								const selected = state.options
								if (!(selected instanceof Set) || selected.size === 0) return header
								const items = await Promise.all(
									[...selected].map(
										(id) => serverSideOptInMsgs[id]?.(state) ?? Promise.resolve(''),
									),
								)
								return `${header}\n${items.join('\n')}`
							}),
					)
					.children(
						group()
							.multiSelect('options')
							.title('Which features require a Server-side Opt-in?')
							.children(
								option('xRay', 'X-ray'),
								option('aimBot', 'Aim Assist'),
								option('movement', 'Movement'),
								option('pvp', 'PvP'),
								option('hidingMods', 'Anti 3.x'),
								option('itemDuplication', 'Dupe'),
							),
					),

				toggle('excessiveLanguages', 'Excessive languages')
					.shown(
						computed(
							() =>
								!!project.value.minecraft_server &&
								!!project.value.minecraft_server?.languages?.length &&
								project.value.minecraft_server.languages.length > 4,
						),
					)
					.action(action().suggestedStatus('flagged').severity('low').message()),

				toggle('ruleBreakingOther', 'Other')
					// TODO: chyz, the required asterisk is on a separate line
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('critical')
							.message((state) => ({ MESSAGE: state.message })),
					)
					.children(
						markdown('message').title('Explain how it infringes on content rules.').required(),
					),
			),
		)
}
