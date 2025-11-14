<template>
	<div
		id="medal"
		class="medal-promotion flex w-full flex-col items-start gap-4 rounded-xl px-4 py-4 shadow-xl md:flex-row md:items-center md:justify-between md:gap-0 md:px-8 md:py-6"
	>
		<MedalBackgroundImage />
		<div
			class="z-10 flex items-start gap-3 text-xl font-semibold text-contrast md:items-center md:gap-6 md:text-2xl"
		>
			<MedalIcon class="h-8 w-auto text-contrast md:h-10" />
			<div class="flex flex-col items-start gap-1">
				<span>
					Try a free
					<span class="text-medal-orange">3GB server</span> for 5 days powered by
					<span class="text-medal-orange">Medal</span>
				</span>
				<span class="text-xs font-medium text-secondary md:text-sm">
					Limited-time offer. No credit card required. Available for US servers.
				</span>
			</div>
		</div>
		<ButtonStyled color="medal-promo" type="outlined" size="large" class="z-10 my-auto mt-2">
			<nuxt-link
				to="https://medal.tv/modrinth"
				class="z-10 flex w-full items-center justify-center gap-1 md:mt-0 md:w-auto"
				>Learn more <ExternalIcon
			/></nuxt-link>
		</ButtonStyled>
	</div>
</template>

<script lang="ts" setup>
import { ExternalIcon } from '@modrinth/assets'
import { ButtonStyled, MedalBackgroundImage } from '@modrinth/ui'

import MedalIcon from '~/assets/images/illustrations/medal_icon.svg?component'
</script>

<style scoped lang="scss">
.medal-promotion {
	position: relative;
	border: 1px solid var(--medal-promotion-bg-orange);
	overflow: hidden;
}

.clock-glow {
	filter: drop-shadow(0 0 72px var(--medal-promotion-bg-orange))
		drop-shadow(0 0 36px var(--medal-promotion-bg-orange))
		drop-shadow(0 0 18px var(--medal-promotion-bg-orange));
}

.text-medal-orange {
	color: var(--medal-promotion-text-orange);
}
</style>
