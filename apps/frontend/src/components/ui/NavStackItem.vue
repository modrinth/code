<template>
	<NuxtLink v-if="link !== null" :to="link" class="nav-item">
		<slot />
		<span>{{ label }}</span>
		<span v-if="badge" class="rounded-full bg-brand-highlight px-2 text-sm font-bold text-brand">{{
			badge
		}}</span>
		<span v-if="chevron" class="ml-auto"><ChevronRightIcon /></span>
	</NuxtLink>
	<button v-else-if="action" class="nav-item" :class="{ 'danger-button': danger }" @click="action">
		<slot />
		<span>{{ label }}</span>
		<span v-if="badge" class="rounded-full bg-brand-highlight px-2 text-sm font-bold text-brand">{{
			badge
		}}</span>
	</button>
	<span v-else>i forgor 💀</span>
</template>

<script>
import { ChevronRightIcon } from '@modrinth/assets'

export default {
	components: {
		ChevronRightIcon,
	},
	props: {
		link: {
			default: null,
			type: String,
		},
		action: {
			default: null,
			type: Function,
		},
		label: {
			required: true,
			type: String,
		},
		badge: {
			default: null,
			type: String,
		},
		chevron: {
			default: false,
			type: Boolean,
		},
		danger: {
			default: false,
			type: Boolean,
		},
	},
}
</script>

<style lang="scss" scoped>
.nav-item {
	@apply flex w-full cursor-pointer items-center gap-2 text-nowrap rounded-xl border-none bg-transparent px-4 py-2 text-left font-semibold text-button-text transition-all hover:bg-button-bg hover:text-contrast active:scale-[0.97];
}

.router-link-exact-active.nav-item {
	@apply bg-button-bgSelected text-button-textSelected;
}
</style>
