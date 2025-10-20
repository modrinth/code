<template>
	<Admonition
		v-if="shouldShowAlert"
		:type="hasHitLimit ? 'critical' : 'warning'"
		:header="
			hasHitLimit
				? capitalizeString(formatMessage(messages.limitReached, { type: typeName.singular }))
				: formatMessage(messages.approachingLimit, { type: typeName.singular, current, max })
		"
		class="mb-4"
	>
		<div class="flex w-full flex-col gap-4">
			<template v-if="hasHitLimit">
				{{ formatMessage(messages.limitReachedDescription, { type: typeName.singular, max }) }}
				<div class="w-min">
					<ButtonStyled color="red">
						<NuxtLink to="https://support.modrinth.com" target="_blank">
							<MessageIcon />{{ formatMessage(messages.contactSupport) }}</NuxtLink
						>
					</ButtonStyled>
				</div>
			</template>
			<template v-else>
				{{
					formatMessage(messages.approachingLimitDescription, {
						type: typeName.singular,
						max,
						typePlural: typeName.plural,
					})
				}}
				<div class="w-min">
					<ButtonStyled color="orange">
						<NuxtLink to="https://support.modrinth.com" target="_blank">
							<MessageIcon />{{ formatMessage(messages.contactSupport) }}</NuxtLink
						>
					</ButtonStyled>
				</div>
			</template>
		</div>
	</Admonition>
</template>

<script setup lang="ts">
import { MessageIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled } from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import { defineMessages } from '@vintl/vintl'
import { computed, watch } from 'vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	limitReached: {
		id: 'create.limit-alert.limit-reached',
		defaultMessage: '{type} limit reached',
	},
	approachingLimit: {
		id: 'create.limit-alert.approaching-limit',
		defaultMessage: 'Approaching {type} limit ({current}/{max})',
	},
	limitReachedDescription: {
		id: 'create.limit-alert.limit-reached-description',
		defaultMessage:
			"You've reached your {type} limit of {max}. Please contact support to increase your limit.",
	},
	approachingLimitDescription: {
		id: 'create.limit-alert.approaching-limit-description',
		defaultMessage:
			"You're about to hit the {type} limit, please contact support if you need more than {max} {typePlural}.",
	},
	contactSupport: {
		id: 'create.limit-alert.contact-support',
		defaultMessage: 'Contact support',
	},
	typeProject: {
		id: 'create.limit-alert.type-project',
		defaultMessage: 'project',
	},
	typeOrganization: {
		id: 'create.limit-alert.type-organization',
		defaultMessage: 'organization',
	},
	typeCollection: {
		id: 'create.limit-alert.type-collection',
		defaultMessage: 'collection',
	},
	typePluralProject: {
		id: 'create.limit-alert.type-plural-project',
		defaultMessage: 'projects',
	},
	typePluralOrganization: {
		id: 'create.limit-alert.type-plural-organization',
		defaultMessage: 'organizations',
	},
	typePluralCollection: {
		id: 'create.limit-alert.type-plural-collection',
		defaultMessage: 'collections',
	},
})

interface UserLimits {
	current: number
	max: number
}

const props = defineProps<{
	type: 'project' | 'org' | 'collection'
}>()

const model = defineModel<boolean>()

const apiEndpoint = computed(() => {
	switch (props.type) {
		case 'project':
			return 'limits/projects'
		case 'org':
			return 'limits/organizations'
		case 'collection':
			return 'limits/collections'
		default:
			return 'limits/projects'
	}
})

const { data: limits } = await useAsyncData<UserLimits | undefined>(
	`limits-${props.type}`,
	() => useBaseFetch(apiEndpoint.value, { apiVersion: 3 }) as Promise<UserLimits>,
)

const typeName = computed<{ singular: string; plural: string }>(() => {
	switch (props.type) {
		case 'project':
			return {
				singular: formatMessage(messages.typeProject),
				plural: formatMessage(messages.typePluralProject),
			}
		case 'org':
			return {
				singular: formatMessage(messages.typeOrganization),
				plural: formatMessage(messages.typePluralOrganization),
			}
		case 'collection':
			return {
				singular: formatMessage(messages.typeCollection),
				plural: formatMessage(messages.typePluralCollection),
			}
		default:
			return {
				singular: formatMessage(messages.typeProject),
				plural: formatMessage(messages.typePluralProject),
			}
	}
})

const current = computed(() => limits.value?.current ?? 0)
const max = computed(() => limits.value?.max ?? null)
const percentage = computed(() => (max.value ? Math.round((current.value / max.value) * 100) : 0))
const hasHitLimit = computed(() => max.value !== null && current.value >= max.value)
const shouldShowAlert = computed(() => max.value !== null && percentage.value >= 75)

watch(
	hasHitLimit,
	(newValue) => {
		model.value = newValue
	},
	{ immediate: true },
)
</script>
