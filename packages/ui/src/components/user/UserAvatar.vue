<template>
	<div class="user-avatar relative inline-flex shrink-0" :style="{ '--_size': cssSize }">
		<div class="flex" :class="{ 'user-avatar-cutout': badge }">
			<Avatar
				:src="src"
				:alt="alt"
				:size="size"
				:loading="loading"
				:tint-by="tintBy"
				:no-shadow="noShadow"
				:raised="raised"
				:class="{ grayscale }"
				circle
			/>
		</div>
		<span v-if="badge" class="user-avatar-badge" aria-hidden="true">
			<slot>
				<span class="block size-full rounded-full bg-brand" />
			</slot>
		</span>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import Avatar from '../base/Avatar.vue'

const props = withDefaults(
	defineProps<{
		src?: string | null
		alt?: string
		size?: string
		badge?: boolean
		grayscale?: boolean
		loading?: 'eager' | 'lazy'
		raised?: boolean
		tintBy?: string | null
		noShadow?: boolean
	}>(),
	{
		src: null,
		alt: '',
		size: '2rem',
		badge: false,
		grayscale: false,
		loading: 'eager',
		raised: false,
		tintBy: null,
		noShadow: true,
	},
)

const LEGACY_PRESETS: Record<string, string> = {
	xxs: '1.25rem',
	xs: '2.5rem',
	sm: '3rem',
	md: '6rem',
	lg: '9rem',
}

const cssSize = computed(() => LEGACY_PRESETS[props.size] ?? props.size)
</script>

<style lang="scss" scoped>
.user-avatar {
	--user-avatar-badge-size: calc(var(--_size) * 11 / 32);
	--user-avatar-badge-gap: max(2px, calc(var(--_size) * 3 / 32));
}

.user-avatar-cutout {
	--cutout: calc(var(--user-avatar-badge-size) / 2 + var(--user-avatar-badge-gap));

	-webkit-mask-image: radial-gradient(
		circle at calc(100% - var(--user-avatar-badge-size) / 2)
			calc(100% - var(--user-avatar-badge-size) / 2),
		transparent var(--cutout),
		#000 calc(var(--cutout) + 0.5px)
	);
	mask-image: radial-gradient(
		circle at calc(100% - var(--user-avatar-badge-size) / 2)
			calc(100% - var(--user-avatar-badge-size) / 2),
		transparent var(--cutout),
		#000 calc(var(--cutout) + 0.5px)
	);
	-webkit-mask-repeat: no-repeat;
	mask-repeat: no-repeat;
	-webkit-mask-size: 100% 100%;
	mask-size: 100% 100%;
}

.user-avatar-badge {
	position: absolute;
	right: 0;
	bottom: 0;
	display: flex;
	width: var(--user-avatar-badge-size);
	height: var(--user-avatar-badge-size);
	pointer-events: none;
}
</style>
