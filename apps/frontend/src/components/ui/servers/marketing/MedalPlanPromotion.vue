<template>
	<div
		id="medal"
		class="medal-promotion flex w-full flex-row justify-between rounded-xl px-8 py-6 shadow-xl"
	>
		<MedalBackgroundImage />
		<div class="z-10 flex items-center gap-6 text-2xl font-semibold text-contrast">
			<MedalIcon class="h-10 w-auto text-contrast" />
			<div class="flex flex-col items-start gap-1">
				<span>
					Try a free
					<span class="text-medal-orange">3GB server</span> for 5 days powered by
					<span class="text-medal-orange">Medal</span>
				</span>
				<span class="text-sm font-medium text-secondary">
					Limited-time offer. No credit card required. Available for US servers.
				</span>
			</div>
		</div>
		<ButtonStyled color="medal-promo" type="outlined" size="large">
			<nuxt-link to="https://medal.tv/modrinth" class="z-10 my-auto"
				>Learn more <ExternalIcon
			/></nuxt-link>
		</ButtonStyled>
	</div>
</template>

<script lang="ts" setup>
import { ExternalIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'

import MedalIcon from '~/assets/images/illustrations/medal_icon.svg?component'
import MedalBackgroundImage from '~/components/ui/servers/marketing/MedalBackgroundImage.vue'
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
