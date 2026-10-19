import { BookOpenIcon } from '@modrinth/assets'

import { messageWithOptionalSuggestion } from '../..'
import minecraftBranding from '../messages/checklist/messages/title-slug/title/minecraft-branding.md'
import similarities from '../messages/checklist/messages/title-slug/title/similarities.md'
import forkSimilarities from '../messages/checklist/messages/title-slug/title/similarities/fork.md'
import modpackSimilarities from '../messages/checklist/messages/title-slug/title/similarities/modpack.md'
import uselessInfo from '../messages/checklist/messages/title-slug/title/useless-info.md'
import { issue, panel, section, text, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'

const panelCategory = 'Title'

export const issueTargetModifyTitleWithOptionalSuggestion = issueTargets.modifyTitle(
	({ getTextValue }) => {
		const suggestion = getTextValue('title-suggestion')
		return suggestion.trim() ? suggestion : undefined
	},
)

export const titleUselessInfoIssue = issue({
	id: 'title-useless-info',
	title: 'Unnecessary title information',
	category: panelCategory,
	actions: [issueTargetModifyTitleWithOptionalSuggestion],
	suggestedStatus: 'flagged',
	message: ({ getTextValue }) =>
		messageWithOptionalSuggestion(uselessInfo, panelCategory, getTextValue('title-suggestion')),
})

export const minecraftTitleIssue = issue({
	id: 'title-minecraft-branding',
	title: 'Minecraft branding in title',
	category: panelCategory,
	actions: [issueTargetModifyTitleWithOptionalSuggestion],
	suggestedStatus: 'flagged',
	message: ({ getTextValue }) =>
		messageWithOptionalSuggestion(
			minecraftBranding,
			panelCategory,
			getTextValue('title-suggestion'),
		),
})

export const titleSimilaritiesIssue = issue({
	id: 'title-similarities',
	title: 'Misuse of project name',
	category: panelCategory,
	actions: [issueTargetModifyTitleWithOptionalSuggestion],
	suggestedStatus: 'flagged',
	message: ({ selected, getTextValue }) => {
		let msg = similarities.trim()

		if (selected.toggleIds.includes('title-similarities-fork')) {
			msg = [msg, forkSimilarities.trim()].join('\n\n')
		}
		if (selected.toggleIds.includes('title-similarities-modpack')) {
			msg = [msg, modpackSimilarities.trim()].join('\n\n')
		}

		return messageWithOptionalSuggestion(msg, panelCategory, getTextValue('title-suggestion'))
	},
})

export const titleReviewPanel = panel({
	title: 'Title',
	hint: "Is the project's name accurate and appropriate?",
	icon: BookOpenIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705',
}).content(
	section().content(
		toggle({
			label: 'Contains Useless Info',
			issue: titleUselessInfoIssue,
		}),
		toggle({
			label: 'Minecraft Title',
			issue: minecraftTitleIssue,
		}),
		toggle({
			label: 'Title Similarities',
			issue: titleSimilaritiesIssue,
		}),
	),
	section({
		label: 'Similarities Additional Info',
		shown: (ctx) => ctx.selected.issueIds.includes('title-similarities'),
	}).content(
		toggle({
			label: 'Forked Project',
			id: 'title-similarities-fork',
			issue: titleSimilaritiesIssue,
			shown: (ctx) => !ctx.projectV3.minecraft_server,
		}),
		toggle({
			label: 'Modpack Named After Mod',
			id: 'title-similarities-modpack',
			issue: titleSimilaritiesIssue,
			shown: (ctx) => ctx.projectV3.project_types.includes('modpack'),
		}),
	),
	section({
		shown: ({ selected }) =>
			selected.issueIds.some((id) =>
				[titleUselessInfoIssue.id, minecraftTitleIssue.id, titleSimilaritiesIssue.id].includes(id),
			),
	}).content(
		text({
			id: 'title-suggestion',
			label: 'Suggestion',
		}),
	),
)
