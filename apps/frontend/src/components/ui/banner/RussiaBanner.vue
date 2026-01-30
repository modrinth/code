<script setup lang="ts">
import { BookTextIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, commonMessages, PagewideBanner, useVIntl } from '@modrinth/ui'

const flags = useFeatureFlags()
const { formatMessage } = useVIntl()

function hideRussiaCensorshipBanner() {
	flags.value.hideRussiaCensorshipBanner = true
	saveFeatureFlags()
}
</script>

<template>
	<PagewideBanner v-if="!flags.hideRussiaCensorshipBanner" variant="error">
		<template #title>
			<div class="flex flex-col gap-1 text-contrast">
				<span lang="ru">К сожалению, Modrinth скоро станет недоступен в России</span>
				<span class="text-sm font-medium opacity-50" lang="en">
					Modrinth will soon be unavailable in Russia
				</span>
			</div>
		</template>
		<template #description>
			<p class="m-0" lang="ru">
				Российское правительство потребовало от нас заблокировать некоторые проекты на Modrinth, но
				мы решили отказать им в цензуре.
			</p>
			<p class="-mt-2 mb-0 text-sm opacity-50" lang="en">
				The Russian government has asked us to censor certain topics on Modrinth and we have decided
				to refuse to comply with their requests.
			</p>

			<p class="m-0 font-semibold" lang="ru">
				Пожалуйста, найдите какой-нибудь надёжный VPN или прокси, чтобы не потерять доступ к
				Modrinth.
			</p>
			<p class="-mt-2 mb-0 text-sm opacity-50" lang="en">
				Please seek a reputable VPN or proxy of some kind to continue to access Modrinth in Russia.
			</p>
		</template>
		<template #actions>
			<div class="mt-2 flex w-fit gap-2">
				<ButtonStyled color="brand">
					<nuxt-link to="/news/article/standing-by-our-values-russian">
						<BookTextIcon /> Прочесть наше полное заявление
						<span class="text-xs font-medium">(Перевод на русский)</span>
					</nuxt-link>
				</ButtonStyled>
				<ButtonStyled>
					<nuxt-link to="/news/article/standing-by-our-values">
						<BookTextIcon /> Read our full statement
						<span class="text-xs font-medium">(English)</span>
					</nuxt-link>
				</ButtonStyled>
			</div>
		</template>
		<template #actions_right>
			<ButtonStyled circular type="transparent">
				<button
					v-tooltip="formatMessage(commonMessages.closeButton)"
					@click="hideRussiaCensorshipBanner"
				>
					<XIcon :aria-label="formatMessage(commonMessages.closeButton)" />
				</button>
			</ButtonStyled>
		</template>
	</PagewideBanner>
</template>
