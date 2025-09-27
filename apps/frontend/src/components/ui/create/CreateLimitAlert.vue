<template>
	<Admonition
		v-if="shouldShowAlert"
		:type="hasHitLimit ? 'critical' : 'warning'"
		:header="
			hasHitLimit
				? formatMessage(messages.limitReached, { type: capitalizeString(typeDisplayName) })
				: formatMessage(messages.approachingLimit, { type: typeDisplayName, current, max })
		"
		class="mb-4"
	>
		<div class="mt-0 flex w-full flex-col gap-2">
			<template v-if="hasHitLimit">
				{{ formatMessage(messages.limitReachedDescription, { type: typeDisplayName, max }) }}
				<div class="w-min">
					<ButtonStyled color="red">
						<NuxtLink to="https://support.modrinth.com" target="_blank">
							<UnknownIcon /> {{ formatMessage(messages.contactSupport) }}</NuxtLink
						>
					</ButtonStyled>
				</div>
			</template>
			<template v-else>
				{{
					formatMessage(messages.approachingLimitDescription, {
						type: typeDisplayName,
						max,
						typePlural: typeDisplayName + 's',
					})
				}}
				<div class="w-min">
					<ButtonStyled color="orange">
						<NuxtLink to="https://support.modrinth.com" target="_blank">
							<UnknownIcon /> {{ formatMessage(messages.contactSupport) }}</NuxtLink
						>
					</ButtonStyled>
				</div>
			</template>
		</div>
	</Admonition>
</template>

<script setup lang="ts">
import { UnknownIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled } from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import { defineMessages } from '@vintl/vintl'
import { computed, ref, watch } from 'vue'

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
		defaultMessage: 'Contact Support',
	},
})

interface LimitsResponse {
	current: {
		projects: number
		orgs: number
		collections: number
	}
	max: {
		projects: number | null
		orgs: number | null
		collections: number | null
	}
}

const props = defineProps<{
	type: 'project' | 'org' | 'collection'
}>()

const model = defineModel<boolean>()

const limits = ref<LimitsResponse>({
	current: {
		projects: 8,
		orgs: 4,
		collections: 12,
	},
	max: {
		projects: 10,
		orgs: 5,
		collections: 15,
	},
})

const limitKey = computed((): keyof LimitsResponse['current'] => {
	switch (props.type) {
		case 'project':
			return 'projects'
		case 'org':
			return 'orgs'
		case 'collection':
			return 'collections'
		default:
			return 'projects'
	}
})

const typeDisplayName = computed(() => {
	switch (props.type) {
		case 'project':
			return 'project'
		case 'org':
			return 'organization'
		case 'collection':
			return 'collection'
		default:
			return 'project'
	}
})

const current = computed(() => limits.value?.current?.[limitKey.value] ?? 0)
const max = computed(() => limits.value?.max?.[limitKey.value] ?? null)
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
