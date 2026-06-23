<script setup lang="ts">
import { computed } from 'vue'

import { TagItem } from '#ui/components'

import { defineMessage, useVIntl } from '../../composables/i18n'

const props = defineProps<{
	variant: 'pending' | 'attributed' | 'no_permission' | 'proof_rejected' | 'not_allowed'
}>()

const { formatMessage } = useVIntl()

const badge = computed(() => {
	switch (props.variant) {
		case 'no_permission':
			return {
				style: { '--_bg-color': 'var(--color-red-bg)', '--_color': 'var(--color-red)' },
				message: defineMessage({
					id: 'external-files.permissions-card.badge.no-permission',
					defaultMessage: 'No permission',
				}),
			}
		case 'proof_rejected':
			return {
				style: { '--_bg-color': 'var(--color-orange-bg)', '--_color': 'var(--color-orange)' },
				message: defineMessage({
					id: 'external-files.permissions-card.badge.proof-rejected',
					defaultMessage: 'Information rejected',
				}),
			}
		case 'not_allowed':
			return {
				style: { '--_bg-color': 'var(--color-red-bg)', '--_color': 'var(--color-red)' },
				message: defineMessage({
					id: 'external-files.permissions-card.badge.not-allowed',
					defaultMessage: 'Not allowed',
				}),
			}
		case 'attributed':
			return {
				style: { '--_bg-color': 'var(--color-green-bg)', '--_color': 'var(--color-green)' },
				message: defineMessage({
					id: 'external-files.permissions-card.badge.attributed',
					defaultMessage: 'Completed',
				}),
			}
		default:
			return {
				style: {
					'--_bg-color': 'var(--color-orange-bg)',
					'--_color': 'var(--color-orange)',
				},
				message: defineMessage({
					id: 'external-files.permissions-card.badge.pending',
					defaultMessage: 'Pending',
				}),
			}
	}
})
</script>

<template>
	<TagItem :style="badge.style">
		{{ formatMessage(badge.message) }}
	</TagItem>
</template>
