<template>
	<div class="card-shadow flex flex-col gap-4 rounded-2xl bg-bg-raised p-4">
		<div class="flex items-center gap-4">
			<div
				class="flex items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg p-2"
			>
				<AutoBrandIcon :keyword="affiliate.source_name" class="h-6 w-6">
					<AffiliateIcon />
				</AutoBrandIcon>
			</div>
			<div class="flex flex-col">
				<span class="w-fit text-lg font-bold text-contrast">
					{{ affiliate.source_name }}
				</span>
				<span v-if="createdBy" class="text-sm text-secondary">
					{{ formatMessage(messages.createdBy, { user: createdBy }) }}
				</span>
			</div>
			<div class="ml-auto flex items-center gap-2">
				<slot />
				<ButtonStyled v-if="showRevoke" color="red" color-fill="text">
					<button @click="emit('revoke', affiliate)">
						<XCircleIcon />
						{{ formatMessage(messages.revokeAffiliateLink) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
		<CopyCode :text="`https://modrinth.gg?afl=${affiliate.id}`" />
	</div>
</template>

<script setup lang="ts">
import { AffiliateIcon, XCircleIcon } from '@modrinth/assets'
import type { AffiliateLink } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'

import { AutoBrandIcon, ButtonStyled, CopyCode } from '../index.ts'

withDefaults(
	defineProps<{
		affiliate: AffiliateLink
		showRevoke?: boolean
		createdBy?: string
	}>(),
	{
		showRevoke: true,
		createdBy: undefined,
	},
)

const emit = defineEmits<{
	(e: 'revoke', affiliate: AffiliateLink): void
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	viewAnalytics: {
		id: 'affiliate.viewAnalytics',
		defaultMessage: 'View analytics',
	},
	revokeAffiliateLink: {
		id: 'affiliate.revoke',
		defaultMessage: 'Revoke affiliate link',
	},
	createdBy: {
		id: 'affiliate.createdBy',
		defaultMessage: 'Created by {user}',
	},
})
</script>
