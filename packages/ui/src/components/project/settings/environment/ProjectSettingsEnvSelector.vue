<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { defineMessage, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { computed, ref, watch } from 'vue'

import { commonProjectSettingsMessages } from '../../../../utils'
import LargeRadioButton from '../../../base/LargeRadioButton.vue'

const { formatMessage } = useVIntl()

const value = defineModel<Labrinth.Projects.v3.Environment | undefined>({ required: true })

withDefaults(
	defineProps<{
		disabled?: boolean
	}>(),
	{
		disabled: false,
	},
)

type EnvironmentRadioOption = {
	title: MessageDescriptor
	description?: MessageDescriptor
}

const subOptionLabel = defineMessage({
	id: 'project.settings.environment.suboption.accessibility-suboption-group-label',
	defaultMessage: 'Suboptions of {option}',
})
const optionLabelFormat = defineMessage({
	id: 'project.settings.environment.suboption.accessibility-option-label',
	defaultMessage: '{title}: {description}',
})

const OUTER_OPTIONS = {
	client: {
		title: defineMessage({
			id: 'project.settings.environment.client_only.title',
			defaultMessage: 'Client-side only',
		}),
		description: defineMessage({
			id: 'project.settings.environment.client_only.description',
			defaultMessage:
				'All functionality is done client-side and is compatible with vanilla servers.',
		}),
		suboptions: {},
	},
	server: {
		title: defineMessage({
			id: 'project.settings.environment.server_only.title',
			defaultMessage: 'Server-side only',
		}),
		description: defineMessage({
			id: 'project.settings.environment.server_only.description',
			defaultMessage:
				'All functionality is done server-side and is compatible with vanilla clients.',
		}),
		suboptions: {
			singleplayer: {
				title: defineMessage({
					id: 'project.settings.environment.server_only.supports_singleplayer.title',
					defaultMessage: 'Works in singleplayer too',
				}),
			},
			dedicated: {
				title: defineMessage({
					id: 'project.settings.environment.server_only.dedicated_only.title',
					defaultMessage: 'Dedicated server only',
				}),
			},
		},
	},
	client_and_server: {
		title: defineMessage({
			id: 'project.settings.environment.client_and_server.title',
			defaultMessage: 'Client and server',
		}),
		description: defineMessage({
			id: 'project.settings.environment.client_and_server.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
		suboptions: {
			required_both: {
				title: defineMessage({
					id: 'project.settings.environment.client_and_server.required_both.title',
					defaultMessage: 'Required on both',
				}),
			},
			optional_client: {
				title: defineMessage({
					id: 'project.settings.environment.client_and_server.optional_client.title',
					defaultMessage: 'Optional on client',
				}),
			},
			optional_server: {
				title: defineMessage({
					id: 'project.settings.environment.client_and_server.optional_server.title',
					defaultMessage: 'Optional on server',
				}),
			},
			optional_both_prefers_both: {
				title: defineMessage({
					id: 'project.settings.environment.client_and_server.optional_both_prefers_both.title',
					defaultMessage: 'Optional on both, works best when installed on both sides',
				}),
			},
			optional_both: {
				title: defineMessage({
					id: 'project.settings.environment.client_and_server.optional_both.title',
					defaultMessage: 'Optional on both, works the same if installed on either side',
				}),
			},
		},
	},
	singleplayer: {
		title: defineMessage({
			id: 'project.settings.environment.singleplayer.title',
			defaultMessage: 'Singleplayer only',
		}),
		description: defineMessage({
			id: 'project.settings.environment.singleplayer.description',
			defaultMessage: `Only functions in Singleplayer or when not connected to a Multiplayer server.`,
		}),
		suboptions: {},
	},
} as const satisfies Record<
	string,
	EnvironmentRadioOption & { suboptions: Record<string, EnvironmentRadioOption> }
>
type OuterOptionKey = keyof typeof OUTER_OPTIONS
type SubOptionKey = ValidKeys<(typeof OUTER_OPTIONS)[keyof typeof OUTER_OPTIONS]['suboptions']>

const currentOuterOption = ref<OuterOptionKey>()
const currentSubOption = ref<SubOptionKey>()

const computedOption = computed<Labrinth.Projects.v3.Environment>(() => {
	switch (currentOuterOption.value) {
		case 'client':
			return 'client_only'
		case 'server':
			switch (currentSubOption.value) {
				case 'singleplayer':
					return 'server_only'
				case 'dedicated':
					return 'dedicated_server_only'
				default:
					return 'unknown'
			}
		case 'client_and_server':
			switch (currentSubOption.value) {
				case 'required_both':
					return 'client_and_server'
				case 'optional_client':
					return 'server_only_client_optional'
				case 'optional_server':
					return 'client_only_server_optional'
				case 'optional_both_prefers_both':
					return 'client_or_server_prefers_both'
				case 'optional_both':
					return 'client_or_server'
				default:
					return 'unknown'
			}
		case 'singleplayer':
			return 'singleplayer_only'
		default:
			return 'unknown'
	}
})

function loadEnvironmentValues(env?: Labrinth.Projects.v3.Environment) {
	switch (env) {
		case 'client_and_server':
			currentOuterOption.value = 'client_and_server'
			currentSubOption.value = 'required_both'
			break
		case 'client_only':
			currentOuterOption.value = 'client'
			currentSubOption.value = undefined
			break
		case 'client_only_server_optional':
			currentOuterOption.value = 'client_and_server'
			currentSubOption.value = 'optional_server'
			break
		case 'singleplayer_only':
			currentOuterOption.value = 'singleplayer'
			currentSubOption.value = undefined
			break
		case 'server_only':
			currentOuterOption.value = 'server'
			currentSubOption.value = 'singleplayer'
			break
		case 'server_only_client_optional':
			currentOuterOption.value = 'client_and_server'
			currentSubOption.value = 'optional_client'
			break
		case 'dedicated_server_only':
			currentOuterOption.value = 'server'
			currentSubOption.value = 'dedicated'
			break
		case 'client_or_server':
			currentOuterOption.value = 'client_and_server'
			currentSubOption.value = 'optional_both'
			break
		case 'client_or_server_prefers_both':
			currentOuterOption.value = 'client_and_server'
			currentSubOption.value = 'optional_both_prefers_both'
			break
		default:
			currentOuterOption.value = undefined
			currentSubOption.value = undefined
			break
	}
}

// Keep parent in sync when local radio selections change
watch(computedOption, (newValue) => {
	if (value.value !== newValue) {
		value.value = newValue
	}
})

// Keep local selections in sync when parent model changes
watch(
	() => value.value,
	(newVal) => {
		loadEnvironmentValues(newVal)
	},
	{ immediate: true },
)

const simulateSave = ref(false)
</script>

<template>
	<div role="radiogroup" :aria-label="formatMessage(commonProjectSettingsMessages.environment)">
		<template
			v-for="({ title, description, suboptions }, key, index) in OUTER_OPTIONS"
			:key="`env-option-${key}`"
		>
			<LargeRadioButton
				class="!w-full"
				:class="{ 'mt-2': index > 0 }"
				:selected="currentOuterOption === key"
				:disabled="disabled"
				:aria-label="
					formatMessage(optionLabelFormat, {
						title: formatMessage(title),
						description: formatMessage(description),
					})
				"
				@select="
					() => {
						if (currentOuterOption !== key) {
							currentSubOption = suboptions
								? (Object.keys(suboptions)[0] as SubOptionKey)
								: undefined
						}
						currentOuterOption = key
						simulateSave = false
					}
				"
			>
				<span class="flex flex-col">
					<span>{{ formatMessage(title) }}</span>
					<span v-if="description" class="text-sm text-secondary">{{
						formatMessage(description)
					}}</span>
				</span>
			</LargeRadioButton>
			<div
				v-if="suboptions"
				class="pl-8"
				role="radiogroup"
				:aria-label="formatMessage(subOptionLabel, { option: formatMessage(title) })"
			>
				<LargeRadioButton
					v-for="(
						{ title: suboptionTitle, description: suboptionDescription }, suboptionKey
					) in suboptions"
					:key="`env-option-${key}-${suboptionKey}`"
					class="!w-full mt-2"
					:class="{
						'opacity-50': currentOuterOption !== key,
					}"
					:selected="currentSubOption === suboptionKey"
					:disabled="disabled"
					@select="
						() => {
							currentOuterOption = key
							currentSubOption = suboptionKey
						}
					"
				>
					<span class="flex flex-col">
						<span>{{ formatMessage(suboptionTitle) }}</span>
						<span v-if="suboptionDescription" class="text-sm text-secondary">{{
							formatMessage(suboptionDescription)
						}}</span>
					</span>
				</LargeRadioButton>
			</div>
		</template>
	</div>
</template>
