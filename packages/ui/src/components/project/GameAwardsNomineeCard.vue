<template>
	<component
		:is="compact ? 'a' : 'section'"
		class="game-awards-nominee-card mb-3 flex flex-col gap-2.5 rounded-[var(--radius-lg)] border border-solid p-4 !border-[color:var(--tga-divider)] bg-[image:var(--tga-background)] text-[--tga-meta]"
		:class="
			compact
				? 'cursor-pointer no-underline transition-[filter] hover:brightness-110 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--tga-accent)]'
				: undefined
		"
		:aria-labelledby="compact ? undefined : titleId"
		v-bind="compact ? { href: link, target: '_blank', rel: 'noopener noreferrer' } : {}"
	>
		<div class="flex gap-2" :class="compact ? 'items-center' : 'items-start'">
			<div class="flex min-w-0 flex-1 items-center gap-2">
				<TheGameAwardsIcon class="size-8 shrink-0 text-[--tga-accent]" aria-hidden="true" />
				<div>
					<p class="m-0 text-sm text-[--tga-accent] font-normal">
						{{ eyebrow }}
					</p>
					<p class="m-0 text-sm font-medium text-[--tga-heading]">
						{{ formatMessage(messages.eventName, { year: String(year) }) }}
					</p>
				</div>
			</div>
			<IconButton
				v-if="!compact"
				v-tooltip="formatMessage(messages.dismiss)"
				type="quiet"
				class="!size-6 shrink-0 !text-[--tga-meta]"
				:label="formatMessage(messages.dismiss)"
				@click="emit('dismiss')"
			>
				<XIcon aria-hidden="true" />
			</IconButton>
		</div>
		<template v-if="!compact">
			<hr class="w-full border-0 border-t border-solid border-t-[--tga-divider]" />
			<div class="flex flex-col gap-1.5">
				<h2 :id="titleId" class="m-0 text-lg font-semibold leading-none text-[--tga-heading]">
					{{ formatMessage(messages.category) }}
				</h2>
				<p class="m-0 text-sm leading-tight">
					{{ formatMessage(messages.body, { projectName }) }}
				</p>
			</div>
			<ButtonLink
				type="colored"
				class="w-full !text-[--tga-button-text] ![--button-color:var(--tga-accent)]"
				:href="link"
				target="_blank"
			>
				<span class="game-awards-nominee-card__vote">{{ formatMessage(messages.vote) }}</span>
				<span class="game-awards-nominee-card__vote-short">{{
					formatMessage(messages.voteNow)
				}}</span>
				<ExternalIcon aria-hidden="true" />
			</ButtonLink>
			<div class="flex items-center gap-1.5 text-sm">
				<CalendarIcon class="mb-[2px] size-4 shrink-0" aria-hidden="true" />
				<span>{{ formatDeadline(votingDeadline) }}</span>
				<span class="size-1.5 shrink-0 rounded-full bg-[--tga-divider]" aria-hidden="true" />
				<span>{{ formatMessage(messages.daysLeft, { days: daysLeft(votingDeadline) }) }}</span>
			</div>
		</template>
	</component>
</template>

<script setup lang="ts">
import { CalendarIcon, ExternalIcon, TheGameAwardsIcon, XIcon } from '@modrinth/assets'
import { computed, useId } from 'vue'

import ButtonLink from '#ui/components/base/buttons/ButtonLink.vue'
import IconButton from '#ui/components/base/buttons/IconButton.vue'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { daysLeft } from '#ui/composables/how-ago'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const props = defineProps<{
	projectName: string
	year: number
	votingDeadline: Date
	link: string
	dismissed?: boolean
}>()

const emit = defineEmits<{
	dismiss: []
}>()

const { formatMessage } = useVIntl()
const formatDeadline = useFormatDateTime({ month: 'short', day: 'numeric' })
const titleId = useId()

const messages = defineMessages({
	officialNominee: {
		id: 'project.game-awards.official-nominee',
		defaultMessage: 'Official nominee',
	},
	eventName: {
		id: 'project.game-awards.event-name',
		defaultMessage: 'The Game Awards {year}',
	},
	category: {
		id: 'project.game-awards.category.best-user-creation',
		defaultMessage: 'Best User Creation',
	},
	nomineeFor: {
		id: 'project.game-awards.nominee-for',
		defaultMessage: 'Nominee for {category}',
	},
	body: {
		id: 'project.game-awards.body',
		defaultMessage: '{projectName} is one of five nominees. Now it needs your vote!',
	},
	vote: {
		id: 'project.game-awards.vote',
		defaultMessage: 'Vote at The Game Awards',
	},
	voteNow: {
		id: 'project.game-awards.vote-now',
		defaultMessage: 'Vote now',
	},
	daysLeft: {
		id: 'project.game-awards.days-left',
		defaultMessage: '{days, plural, one {# day left} other {# days left}}',
	},
	dismiss: {
		id: 'project.game-awards.hide-voting-details',
		defaultMessage: 'Hide voting details',
	},
})

const compact = computed(
	() => props.dismissed === true || props.votingDeadline.getTime() <= Date.now(),
)

const eyebrow = computed(() => {
	if (compact.value) {
		return formatMessage(messages.nomineeFor, {
			category: formatMessage(messages.category),
		})
	}

	return formatMessage(messages.officialNominee)
})
</script>

<style>
.game-awards-nominee-card {
	--tga-accent: #20a4b7;
	--tga-heading: var(--color-contrast);
	--tga-meta: var(--color-secondary);
	--tga-button-text: #ffffff;
	--tga-divider: #d5eef3;
	--tga-background: linear-gradient(to bottom right, #ffffff 0%, #e8fbff 100%);
}

.game-awards-nominee-card__vote-short {
	display: none;
}

@media (max-width: 349px) {
	.game-awards-nominee-card__vote {
		display: none;
	}

	.game-awards-nominee-card__vote-short {
		display: inline;
	}
}

html.dark-mode .game-awards-nominee-card,
html.oled-mode .game-awards-nominee-card,
html.retro-mode .game-awards-nominee-card {
	--tga-accent: #ff9f59;
	--tga-heading: #daede5;
	--tga-meta: #afc6c2;
	--tga-button-text: #1a1009;
	--tga-divider: #0c4656;
	--tga-background: linear-gradient(to bottom right, #022e3b 0%, #021418 100%);
}
</style>
