import type { Project } from '@modrinth/utils'

import type {
	Action,
	AdditionalTextInput,
	ButtonAction,
	ConditionalMessage,
	ToggleAction,
} from './types/actions'

export interface ActionState {
	selected: boolean
	value?: Set<number> | number | string | unknown
}

export interface MessagePart {
	weight: number
	content: string
	actionId: string
	stageIndex: number
}

export type SerializedActionState = {
	isSet?: boolean
} & ActionState

export function getActionIdForStage(
	action: Action,
	stageIndex: number,
	actionIndex?: number,
	enabledIndex?: number,
): string {
	if (action.id) {
		return `stage-${stageIndex}-${action.id}`
	}
	const suffix = enabledIndex !== undefined ? `-enabled-${enabledIndex}` : ''
	return `stage-${stageIndex}-action-${actionIndex}${suffix}`
}

export function getActionId(action: Action, currentStage: number, index?: number): string {
	return getActionIdForStage(action, currentStage, index)
}

export function getActionKey(
	action: Action,
	currentStage: number,
	visibleActions: Action[],
): string {
	const index = visibleActions.indexOf(action)
	return `${currentStage}-${index}-${getActionId(action, currentStage)}`
}

export function serializeActionStates(states: Record<string, ActionState>): string {
	const serializable: Record<string, SerializedActionState> = {}
	for (const [key, state] of Object.entries(states)) {
		serializable[key] = {
			selected: state.selected,
			value: state.value instanceof Set ? Array.from(state.value) : state.value,
			isSet: state.value instanceof Set,
		}
	}
	return JSON.stringify(serializable)
}

export function deserializeActionStates(data: string): Record<string, ActionState> {
	try {
		const parsed = JSON.parse(data)
		const states: Record<string, ActionState> = {}
		for (const [key, state] of Object.entries(parsed as Record<string, SerializedActionState>)) {
			states[key] = {
				selected: state.selected,
				value: state.isSet ? new Set(state.value as unknown[]) : state.value,
			}
		}
		return states
	} catch {
		return {}
	}
}

export function initializeActionState(action: Action): ActionState {
	if (action.type === 'toggle') {
		return {
			selected: action.defaultChecked || false,
		}
	} else if (action.type === 'dropdown') {
		return {
			selected: true,
			value: action.defaultOption || 0,
		}
	} else if (action.type === 'multi-select-chips') {
		return {
			selected: false,
			value: new Set<number>(),
		}
	} else {
		return {
			selected: false,
		}
	}
}

export function processMessage(
	message: string,
	action: Action,
	stageIndex: number,
	textInputValues: Record<string, string>,
): string {
	let processedMessage = message

	if (action.relevantExtraInput) {
		action.relevantExtraInput.forEach((input, index) => {
			if (input.variable) {
				const inputKey = `stage-${stageIndex}-${action.id || `action-${index}`}-${index}`
				const value = textInputValues[inputKey] || ''

				const regex = new RegExp(`%${input.variable}%`, 'g')
				processedMessage = processedMessage.replace(regex, value)
			}
		})
	}

	return processedMessage
}

export function findMatchingVariant(
	variants: ConditionalMessage[],
	selectedActionIds: string[],
	allValidActionIds?: string[],
	currentStageIndex?: number,
): ConditionalMessage | null {
	for (const variant of variants) {
		const conditions = variant.conditions

		const meetsRequired =
			!conditions.requiredActions ||
			conditions.requiredActions.every((id) => {
				let fullId = id
				if (currentStageIndex !== undefined && !id.startsWith('stage-')) {
					fullId = `stage-${currentStageIndex}-${id}`
				}

				if (allValidActionIds && !allValidActionIds.includes(fullId)) {
					return false
				}
				return selectedActionIds.includes(fullId)
			})

		const meetsExcluded =
			!conditions.excludedActions ||
			!conditions.excludedActions.some((id) => {
				let fullId = id
				if (currentStageIndex !== undefined && !id.startsWith('stage-')) {
					fullId = `stage-${currentStageIndex}-${id}`
				}
				return selectedActionIds.includes(fullId)
			})

		if (meetsRequired && meetsExcluded) {
			return variant
		}
	}

	return null
}

export async function getActionMessage(
	action: ButtonAction | ToggleAction,
	selectedActionIds: string[],
	allValidActionIds?: string[],
): Promise<string> {
	if (action.conditionalMessages && action.conditionalMessages.length > 0) {
		const matchingConditional = findMatchingVariant(
			action.conditionalMessages,
			selectedActionIds,
			allValidActionIds,
		)
		if (matchingConditional) {
			return (await matchingConditional.message()) as string
		}
	}

	return (await action.message()) as string
}

export function getVisibleInputs(
	action: Action,
	actionStates: Record<string, ActionState>,
): AdditionalTextInput[] {
	if (!action.relevantExtraInput) return []

	const selectedActionIds = Object.entries(actionStates)
		.filter(([, state]) => state.selected)
		.map(([id]) => id)

	return action.relevantExtraInput.filter((input) => {
		if (!input.showWhen) return true

		const meetsRequired =
			!input.showWhen.requiredActions ||
			input.showWhen.requiredActions.every((id) => selectedActionIds.includes(id))

		const meetsExcluded =
			!input.showWhen.excludedActions ||
			!input.showWhen.excludedActions.some((id) => selectedActionIds.includes(id))

		return meetsRequired && meetsExcluded
	})
}

export function expandVariables(
	template: string,
	project: Project,
	variables?: Record<string, string>,
): string {
	if (!variables) {
		variables = flattenProjectVariables(project)
	}

	return Object.entries(variables).reduce((result, [key, value]) => {
		const variable = `%${key}%`
		return result.replace(new RegExp(variable, 'g'), value)
	}, template)
}

export function kebabToTitleCase(input: string): string {
	return input
		.split('-')
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(' ')
}

export function arrayOrNone(arr: string[]): string {
	return arr.length > 0 ? arr.join(', ') : 'None'
}

export function flattenProjectVariables(project: Project): Record<string, string> {
	const vars: Record<string, string> = {}

	vars['PROJECT_ID'] = project.id
	vars['PROJECT_TYPE'] = project.project_type
	vars['PROJECT_SLUG'] = project.slug
	vars['PROJECT_TITLE'] = project.title
	vars['PROJECT_SUMMARY'] = project.description
	vars['PROJECT_STATUS'] = project.status
	vars['PROJECT_REQUESTED_STATUS'] = project.requested_status
	vars['PROJECT_MONETIZATION_STATUS'] = project.monetization_status
	vars['PROJECT_BODY'] = project.body

	vars['PROJECT_ICON_URL'] = project.icon_url || ''
	vars['PROJECT_ISSUES_URL'] = project.issues_url || 'None'
	vars['PROJECT_SOURCE_URL'] = project.source_url || 'None'
	vars['PROJECT_WIKI_URL'] = project.wiki_url || 'None'
	vars['PROJECT_DISCORD_URL'] = project.discord_url || 'None'

	vars['PROJECT_DOWNLOADS'] = project.downloads.toString()
	vars['PROJECT_FOLLOWERS'] = project.followers.toString()
	vars['PROJECT_COLOR'] = project.color?.toString() || ''

	vars['PROJECT_CLIENT_SIDE'] = project.client_side
	vars['PROJECT_SERVER_SIDE'] = project.server_side

	vars['PROJECT_TEAM'] = project.team || 'None'
	vars['PROJECT_THREAD_ID'] = project.thread_id
	vars['PROJECT_ORGANIZATION'] = project.organization

	vars['PROJECT_PUBLISHED'] = project.published
	vars['PROJECT_UPDATED'] = project.updated
	vars['PROJECT_APPROVED'] = project.approved
	vars['PROJECT_QUEUED'] = project.queued

	vars['PROJECT_LICENSE_ID'] = project.license.id
	vars['PROJECT_LICENSE_NAME'] = project.license.name
	vars['PROJECT_LICENSE_URL'] = project.license.url || 'None'

	vars['PROJECT_CATEGORIES'] = arrayOrNone(project.categories)
	vars['PROJECT_ADDITIONAL_CATEGORIES'] = arrayOrNone(project.additional_categories)
	vars['PROJECT_GAME_VERSIONS'] = arrayOrNone(project.game_versions)
	vars['PROJECT_LOADERS'] = arrayOrNone(project.loaders)
	vars['PROJECT_VERSIONS'] = arrayOrNone(project.versions)

	vars['PROJECT_CATEGORIES_COUNT'] = project.categories.length.toString()
	vars['PROJECT_GAME_VERSIONS_COUNT'] = project.game_versions.length.toString()
	vars['PROJECT_LOADERS_COUNT'] = project.loaders.length.toString()
	vars['PROJECT_VERSIONS_COUNT'] = project.versions.length.toString()
	vars['PROJECT_GALLERY_COUNT'] = (project.gallery?.length || 0).toString()
	vars['PROJECT_DONATION_URLS_COUNT'] = project.donation_urls.length.toString()

	project.donation_urls.forEach((donation, index) => {
		vars[`PROJECT_DONATION_${index}_ID`] = donation.id
		vars[`PROJECT_DONATION_${index}_PLATFORM`] = donation.platform
		vars[`PROJECT_DONATION_${index}_URL`] = donation.url
	})

	project.gallery?.forEach((image, index) => {
		vars[`PROJECT_GALLERY_${index}_URL`] = image.url
		vars[`PROJECT_GALLERY_${index}_TITLE`] = image.title || ''
		vars[`PROJECT_GALLERY_${index}_DESCRIPTION`] = image.description || ''
		vars[`PROJECT_GALLERY_${index}_FEATURED`] = image.featured.toString()
	})

	// Static time saving stuff
	vars[`RULES`] = `[Modrinth's Content Rules](https://modrinth.com/legal/rules)`
	vars[`TOS`] = `[Terms of Use](https://modrinth.com/legal/terms)`
	vars[`COPYRIGHT_POLICY`] = `[Copyright Policy](https://modrinth.com/legal/copyright)`
	vars[`SUPPORT`] =
		`please visit the [Modrinth Help Center](https://support.modrinth.com/) and click the green bubble to contact support.`
	vars[`MODPACK_PERMISSIONS_GUIDE`] =
		`our guide to [Obtaining Modpack Permissions](https://support.modrinth.com/en/articles/8797527-obtaining-modpack-permissions)`
	vars[`MODPACKS_ON_MODRINTH`] =
		`[Modpacks on Modrinth](https://support.modrinth.com/en/articles/8802250-modpacks-on-modrinth)`
	vars[`ADVANCED_MARKDOWN`] =
		`[Markdown Formatting Guide](https://support.modrinth.com/en/articles/8801962-advanced-markdown-formatting)`
	vars[`LICENSING_GUIDE`] =
		`our guide to [Licensing your Mods](https://modrinth.com/news/article/licensing-guide)`
	vars[`NEW_ENVIRONMENTS_LINK`] = `https://modrinth.com/news/article/new-environments`

	// Navigation related variables
	vars[`PROJECT_PERMANENT_LINK`] = `https://modrinth.com/project/${project.id}`
	vars[`PROJECT_SETTINGS_LINK`] = `https://modrinth.com/project/${project.id}/settings`
	vars[`PROJECT_SETTINGS_FLINK`] = `[Settings](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_TITLE_FLINK`] = `[Name](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_SLUG_FLINK`] = `[URL](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_SUMMARY_FLINK`] = `[Summary](https://modrinth.com/project/${project.id}/settings)`
	vars[`PROJECT_ENVIRONMENT_FLINK`] =
		`[Environment Information](https://modrinth.com/project/${project.id}/settings/environment)`
	vars[`PROJECT_TAGS_LINK`] = `https://modrinth.com/project/${project.id}/settings/tags`
	vars[`PROJECT_TAGS_FLINK`] = `[Tags](https://modrinth.com/project/${project.id}/settings/tags)`
	vars[`PROJECT_DESCRIPTION_LINK`] =
		`https://modrinth.com/project/${project.id}/settings/description`
	vars[`PROJECT_DESCRIPTION_FLINK`] =
		`[Description](https://modrinth.com/project/${project.id}/settings/description)`
	vars[`PROJECT_LICENSE_LINK`] = `https://modrinth.com/project/${project.id}/license`
	vars[`PROJECT_LICENSE_FLINK`] = `[License](https://modrinth.com/project/${project.id}/license)`
	vars[`PROJECT_LINKS_LINK`] = `https://modrinth.com/project/${project.id}/settings/links`
	vars[`PROJECT_LINKS_FLINK`] =
		`[External Links](https://modrinth.com/project/${project.id}/settings/links)`
	vars[`PROJECT_GALLERY_LINK`] = `https://modrinth.com/project/${project.id}/gallery`
	vars[`PROJECT_GALLERY_FLINK`] = `[Gallery](https://modrinth.com/project/${project.id}/gallery)`
	vars[`PROJECT_VERSIONS_LINK`] = `https://modrinth.com/project/${project.id}/versions`
	vars[`PROJECT_VERSIONS_FLINK`] = `[Versions](https://modrinth.com/project/${project.id}/versions)`
	vars[`PROJECT_MODERATION_LINK`] = `https://modrinth.com/project/${project.id}/moderation`
	vars[`PROJECT_MODERATION_FLINK`] =
		`[moderation tab](https://modrinth.com/project/${project.id}/moderation)`

	return vars
}
