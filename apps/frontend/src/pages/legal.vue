<template>
	<div class="normal-page">
		<div class="normal-page__sidebar">
			<NavStack
				:items="[
					{ type: 'heading', label: 'Platform' },
					{ link: '/legal/terms', label: 'Terms of Use', icon: HeartHandshakeIcon },
					{ link: '/legal/rules', label: 'Content Rules', icon: ScaleIcon },
					{ link: '/legal/copyright', label: 'Copyright Policy', icon: CopyrightIcon },
					{ link: '/legal/security', label: 'Security Notice', icon: ShieldIcon },
					{ type: 'heading', label: 'Privacy' },
					{ link: '/legal/privacy', label: 'Privacy Policy', icon: LockIcon },
					{ link: '/legal/ccpa', label: 'California Privacy Notice', icon: InfoIcon },
					{ type: 'heading', label: 'Rewards Program' },
					{ link: '/legal/cmp', label: 'Rewards Program Terms', icon: CurrencyIcon },
					{ link: '/legal/cmp-info', label: 'Rewards Program Info', icon: InfoIcon },
				]"
			/>
		</div>
		<div class="normal-page__content">
			<NuxtPage class="universal-card" :route="route" />
		</div>
	</div>
</template>

<script setup>
import {
	CopyrightIcon,
	CurrencyIcon,
	HeartHandshakeIcon,
	InfoIcon,
	LockIcon,
	ScaleIcon,
	ShieldIcon,
} from '@modrinth/assets'

import NavStack from '~/components/ui/NavStack.vue'

const route = useNativeRoute()
</script>

<style lang="scss" scoped>
.normal-page__content :deep(a) {
	color: var(--color-link);
	text-decoration: underline;

	&:focus-visible,
	&:hover {
		color: var(--color-link-hover);
	}

	&:active {
		color: var(--color-link-active);
	}
}
</style>
