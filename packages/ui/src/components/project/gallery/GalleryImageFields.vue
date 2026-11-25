<script setup lang="ts">
import { ImageIcon, StarIcon, TransferIcon } from '@modrinth/assets'
import { useId } from 'vue'

import { useVIntl } from '../../../composables/i18n'
import Button from '../../base/buttons/Button.vue'
import FileButton from '../../base/buttons/FileButton.vue'
import Field from '../../base/Field.vue'
import Input from '../../base/inputs/Input.vue'
import Textarea from '../../base/inputs/Textarea.vue'
import { galleryEditorMessages as messages } from './messages'

defineProps<{
	previewSrc?: string | null
	filename?: string
	accept: string
	maxSize?: number | null
	replaceDisabled?: boolean
}>()

const title = defineModel<string>('title', { required: true })
const description = defineModel<string>('description', { required: true })
const featured = defineModel<boolean>('featured', { required: true })
const emit = defineEmits<{ replace: [files: File[]] }>()
const { formatMessage } = useVIntl()
const titleId = useId()
const descriptionId = useId()
const bannerDescriptionId = useId()
</script>

<template>
	<div class="flex min-w-0 flex-col gap-4">
		<div class="overflow-hidden rounded-xl border border-solid border-surface-4">
			<div v-if="filename" class="flex flex-wrap items-center gap-2 bg-surface-3 p-3">
				<ImageIcon aria-hidden="true" class="size-4 shrink-0" />
				<strong class="min-w-0 flex-1 break-words">{{ filename }}</strong>
				<FileButton
					:disabled="replaceDisabled"
					type="outlined"
					:prompt="formatMessage(messages.replace)"
					:accept="accept"
					:max-size="maxSize"
					:aria-label="formatMessage(messages.replaceImage)"
					@change="emit('replace', $event)"
				>
					<TransferIcon aria-hidden="true" />
				</FileButton>
			</div>
			<img
				class="block max-h-60 w-full bg-surface-1 object-contain"
				:src="previewSrc || 'https://cdn.modrinth.com/placeholder-banner.svg'"
				:alt="formatMessage(messages.previewAlt)"
			/>
		</div>
		<Field :label="formatMessage(messages.imageTitle)" :input-id="titleId">
			<Input
				:id="titleId"
				v-model="title"
				:maxlength="64"
				:placeholder="formatMessage(messages.titlePlaceholder)"
				wrapper-class="w-full"
			/>
			<template #bottom>
				<div v-if="$slots['title-feedback']" class="flex flex-col gap-2 empty:hidden">
					<slot name="title-feedback" />
				</div>
			</template>
		</Field>
		<Field :label="formatMessage(messages.description)" :input-id="descriptionId">
			<Textarea
				:id="descriptionId"
				v-model="description"
				:maxlength="255"
				:placeholder="formatMessage(messages.descriptionPlaceholder)"
			/>
			<template #bottom>
				<div v-if="$slots['description-feedback']" class="flex flex-col gap-2 empty:hidden">
					<slot name="description-feedback" />
				</div>
			</template>
		</Field>
		<div class="flex flex-col gap-2">
			<div class="flex flex-col gap-1">
				<p class="m-0 font-semibold text-contrast">{{ formatMessage(messages.banner) }}</p>
				<p :id="bannerDescriptionId" class="m-0 leading-tight text-secondary">
					{{ formatMessage(messages.bannerDescription) }}
				</p>
			</div>
			<Button
				class="w-fit aria-pressed:!bg-brand-highlight"
				:type="featured ? 'outlined' : 'base'"
				:color="featured ? 'brand' : undefined"
				native-type="button"
				:aria-pressed="featured"
				:aria-label="formatMessage(messages.setBanner)"
				:aria-describedby="bannerDescriptionId"
				@click="featured = !featured"
			>
				<StarIcon :class="{ 'fill-current': featured }" aria-hidden="true" />
				{{ formatMessage(featured ? messages.unsetBanner : messages.setBanner) }}
			</Button>
		</div>
	</div>
</template>
