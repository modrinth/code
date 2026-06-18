<template>
	<div class="mr-2.5 flex min-w-0 items-center gap-2">
		<span
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
		<div class="flex min-w-0 items-center gap-1.5">
			<span
				v-tooltip="labelTooltip"
				class="min-w-0 truncate font-semibold leading-tight text-primary"
			>
				{{ label }}
			</span>
			<OrganizationIcon
				v-if="organizationTooltip"
				v-tooltip="organizationTooltip"
				class="size-4 shrink-0 text-primary"
			/>
		</div>
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
	labelTooltip?: string
	organizationTooltip?: string
}>()

const { formatMessage } = useVIntl()
</script>
