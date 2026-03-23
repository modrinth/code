<script setup lang="ts">
import type { VersionChannel } from '@modrinth/utils'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'

const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		channel: VersionChannel
		/** @deprecated Use size="lg" instead */
		large?: boolean
		size?: 'xs' | 'sm' | 'md' | 'lg'
	}>(),
	{
		large: false,
		size: 'md',
	},
)

const sizeClasses = computed(() => {
	if (props.large) return 'text-lg w-[2.625rem] h-[2.625rem]'
	switch (props.size) {
		case 'xs':
			return 'text-xs w-5 h-5'
		case 'sm':
			return 'text-xs w-7 h-7'
		case 'lg':
			return 'text-lg w-[2.625rem] h-[2.625rem]'
		default:
			return 'text-sm w-9 h-9'
	}
})

const messages = defineMessages({
	releaseSymbol: {
		id: 'project.versions.channel.release.symbol',
		defaultMessage: 'R',
	},
	betaSymbol: {
		id: 'project.versions.channel.beta.symbol',
		defaultMessage: 'B',
	},
	alphaSymbol: {
		id: 'project.versions.channel.alpha.symbol',
		defaultMessage: 'A',
	},
})
</script>

<template>
	<div
		:class="[
			'flex font-bold justify-center items-center rounded-full',
			sizeClasses,
			channel === 'release'
				? 'bg-bg-green text-brand-green'
				: channel === 'beta'
					? 'bg-bg-orange text-brand-orange'
					: 'bg-bg-red text-brand-red',
		]"
	>
		{{ channel ? formatMessage(messages[`${channel}Symbol`]) : '?' }}
	</div>
</template>
