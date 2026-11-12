<script setup lang="ts">
import { Button, ButtonLink, IconButton } from '@modrinth/ui'
import type { RouteLocationRaw } from 'vue-router'

withDefaults(
	defineProps<{
		label: string
		to?: RouteLocationRaw
		active?: boolean
		iconOnly?: boolean
	}>(),
	{
		to: undefined,
		active: false,
		iconOnly: true,
	},
)
</script>

<template>
	<ButtonLink
		v-if="to"
		:to="to"
		type="quiet"
		size="xl"
		class="!w-12 !rounded-full !px-0"
		:class="{ '!bg-button-bgSelected !text-button-textSelected': active }"
		:title="label"
		:aria-label="label"
	>
		<slot />
	</ButtonLink>
	<IconButton
		v-else-if="iconOnly"
		:label="label"
		type="quiet"
		size="xl"
		:class="{ '!bg-button-bgSelected !text-button-textSelected': active }"
		:title="label"
	>
		<slot />
	</IconButton>
	<Button
		v-else
		type="quiet"
		size="xl"
		:class="{ '!bg-button-bgSelected !text-button-textSelected': active }"
	>
		<slot />
	</Button>
</template>
