<template>
	<div class="wrapper relative mb-3 flex w-full justify-center rounded-2xl">
		<AutoLink
			:to="currentAd.link"
			:aria-label="currentAd.description"
			class="flex max-h-[250px] min-h-[250px] min-w-[300px] max-w-[300px] flex-col gap-4 rounded-[inherit] bg-bg-raised"
		>
			<img
				:src="currentAd.light"
				aria-hidden="true"
				:alt="currentAd.description"
				class="light-image hidden rounded-[inherit]"
			/>
			<img
				:src="currentAd.dark"
				aria-hidden="true"
				:alt="currentAd.description"
				class="dark-image rounded-[inherit]"
			/>
		</AutoLink>
		<div
			class="absolute top-0 flex items-center justify-center overflow-hidden rounded-2xl bg-bg-raised"
		>
			<div id="modrinth-rail-1" />
		</div>
	</div>
</template>
<script setup>
import { AutoLink } from '@modrinth/ui'

const flags = useFeatureFlags()

useHead({
	script: [
		// {
		//   // Clean.io
		//   src: "https://cadmus.script.ac/d14pdm1b7fi5kh/script.js",
		// },
		{
			// Aditude
			src: 'https://dn0qt3r0xannq.cloudfront.net/modrinth-7JfmkEIXEp/modrinth-longform/prebid-load.js',
			async: true,
		},
		{
			// Optima
			src: 'https://bservr.com/o.js?uid=8118d1fdb2e0d6f32180bd27',
			async: true,
		},
		{
			src: '/inmobi.js',
			async: true,
		},
	],
	link: [
		{
			rel: 'preload',
			as: 'script',
			href: 'https://www.googletagservices.com/tag/js/gpt.js',
		},
	],
})

const AD_PRESETS = {
	medal: {
		light: 'https://cdn-raw.modrinth.com/medal-modrinth-servers-light-new.webp',
		dark: 'https://cdn-raw.modrinth.com/medal-modrinth-servers-dark-new.webp',
		description: 'Host your next server with Modrinth Servers',
		link: '/servers?plan&ref=medal',
	},
	'modrinth-servers': {
		light: 'https://cdn-raw.modrinth.com/modrinth-servers-placeholder-light.webp',
		dark: 'https://cdn-raw.modrinth.com/modrinth-servers-placeholder-dark.webp',
		description: 'Host your next server with Modrinth Servers',
		link: '/servers',
	},
}

const currentAd = computed(() =>
	flags.value.enableMedalPromotion ? AD_PRESETS.medal : AD_PRESETS['modrinth-servers'],
)

onMounted(() => {
	window.tude = window.tude || { cmd: [] }
	window.Raven = window.Raven || { cmd: [] }

	window.Raven.cmd.push(({ config }) => {
		config.setCustom({
			param1: 'web',
		})
	})

	tude.cmd.push(function () {
		tude.refreshAdsViaDivMappings([
			{
				divId: 'modrinth-rail-1',
				baseDivId: 'pb-slot-square-2',
				targeting: {
					location: 'web',
				},
			},
		])
	})
})
</script>
<style>
iframe[id^='google_ads_iframe'] {
	color-scheme: normal;
	background: transparent;
}

#qc-cmp2-ui {
	background: var(--color-raised-bg);
	border-radius: var(--radius-lg);
	color: var(--color-base);
}

#qc-cmp2-ui::before {
	background: var(--color-raised-bg);
}

#qc-cmp2-ui::after {
	background: var(--color-raised-bg);
}

#qc-cmp2-ui button[mode='primary'] {
	background: var(--color-brand);
	color: var(--color-accent-contrast);
	border-radius: var(--radius-lg);
	border: none;
}

#qc-cmp2-ui button[mode='secondary'] {
	background: var(--color-button-bg);
	color: var(--color-base);
	border-radius: var(--radius-lg);
	border: none;
}

#qc-cmp2-ui button[mode='link'] {
	color: var(--color-link);
}

#qc-cmp2-ui h2 {
	color: var(--color-contrast);
	font-size: 1.5rem;
}

#qc-cmp2-ui div,
#qc-cmp2-ui li,
#qc-cmp2-ui strong,
#qc-cmp2-ui p,
#qc-cmp2-ui .qc-cmp2-list-item-title,
#qc-cmp2-ui .qc-cmp2-expandable-info {
	color: var(--color-base);
	font-family: var(--font-standard);
}

#qc-cmp2-ui .qc-cmp2-toggle[aria-checked='true'] {
	background-color: var(--color-brand);
	border: 1px solid var(--color-brand);
}

@media (max-width: 1024px) {
	.wrapper {
		display: none;
	}
}

.wrapper > * {
	box-shadow: var(--shadow-card);
}
</style>

<style lang="scss" scoped>
.light,
.light-mode {
	.dark-image {
		display: none;
	}

	.light-image {
		display: block;
	}
}
</style>
