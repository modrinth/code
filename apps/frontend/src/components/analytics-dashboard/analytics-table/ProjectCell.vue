<template>
	<div class="mr-2.5 flex min-w-0 items-center gap-2">
		<span
			v-if="!hideIcon"
			v-tooltip="iconTooltip"
			class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
		>
			<img
				v-if="iconUrl"
				:src="iconUrl"
				:alt="formatMessage(analyticsMessages.projectIconAlt, { name: label })"
				class="h-6 w-6 rounded object-cover"
			/>
			<BoxIcon v-else class="h-full w-full" />
		</span>
		<component
			:is="labelHref ? 'a' : 'span'"
			v-tooltip="labelTooltip"
			:href="labelHref"
			:target="labelHref ? '_blank' : undefined"
			:rel="labelHref ? 'noopener noreferrer' : undefined"
			class="min-w-0 truncate font-semibold leading-tight text-primary"
			:class="{ 'hover:underline': labelHref }"
		>
			{{ label }}
		</component>
		<component
			:is="organizationHref ? 'a' : 'span'"
			v-if="organizationTooltip"
			v-tooltip="organizationTooltip"
			:href="organizationHref"
			:target="organizationHref ? '_blank' : undefined"
			:rel="organizationHref ? 'noopener noreferrer' : undefined"
			:aria-label="organizationTooltip"
			class="flex size-4 shrink-0 items-center text-primary"
			:class="{ 'hover:underline': organizationHref }"
		>
			<OrganizationIcon class="size-4" />
		</component>
	</div>
</template>

<script setup lang="ts">
import { BoxIcon, OrganizationIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'

import { analyticsMessages } from '../analytics-messages.ts'

defineProps<{
	label: string
	iconUrl?: string
	iconTooltip?: string
	hideIcon?: boolean
	labelHref?: string
	labelTooltip?: string
	organizationHref?: string
	organizationTooltip?: string
}>()

const { formatMessage } = useVIntl()
</script>
