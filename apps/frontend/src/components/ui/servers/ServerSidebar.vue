<template>
	<div class="static w-full grid-cols-1 md:relative md:flex">
		<div class="static h-full flex-col pb-4 md:flex md:pb-0 md:pr-4">
			<NavStack
				class="z-10 select-none md:w-[16rem]"
				aria-label="Server options"
				:items="navStackItems"
			/>
		</div>

		<div class="h-full w-full">
			<NuxtPage :route="route" @reinstall="onReinstall" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed } from 'vue'
import type { RouteLocationNormalized } from 'vue-router'

import NavStack, { type NavStackEntry } from '~/components/ui/NavStack.vue'

const emit = defineEmits(['reinstall'])

const props = defineProps<{
	navLinks: {
		label: string
		href: string
		icon: Component
		external?: boolean
		shown?: boolean
	}[]
	route: RouteLocationNormalized
}>()

const navStackItems = computed<NavStackEntry[]>(() =>
	props.navLinks.map((link) => ({
		label: link.label,
		link: link.href,
		icon: link.icon,
		chevron: link.external,
		shown: link.shown,
	})),
)

const onReinstall = (...args: any[]) => {
	emit('reinstall', ...args)
}
</script>
