<template>
	<div class="flex flex-col gap-6">
		<template v-if="!ctx.curseforgeModpackFile.value">
			<div class="flex flex-col gap-2">
				<label :for="linkId" class="font-semibold text-contrast">
					{{ formatMessage(messages.linkLabel) }}
				</label>
				<Input
					:id="linkId"
					v-model="ctx.curseforgeUrl.value"
					:icon="LinkIcon"
					type="url"
					size="medium"
					input-class="text-sm"
					placeholder="https://www.curseforge.com/modpacks/all-the-mods-10"
					:error="invalidUrl"
					:aria-describedby="`${linkId}-description`"
				/>
				<p
					:id="`${linkId}-description`"
					class="m-0 leading-6"
					:class="invalidUrl ? 'text-red' : 'text-primary'"
				>
					{{ formatMessage(invalidUrl ? messages.invalidLink : messages.linkDescription) }}
				</p>
			</div>
			<div class="flex items-center gap-4">
				<div class="h-px flex-1 bg-surface-5" />
				<span class="font-medium text-primary">{{ formatMessage(messages.orUpload) }}</span>
				<div class="h-px flex-1 bg-surface-5" />
			</div>
		</template>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				{{
					formatMessage(
						ctx.curseforgeModpackFile.value ? messages.selectedFileLabel : messages.fileLabel,
					)
				}}
			</span>
			<ArchiveInput v-model="ctx.curseforgeModpackFile.value" />
			<p v-if="!ctx.curseforgeModpackFile.value" class="m-0 leading-6 text-primary">
				{{ formatMessage(messages.fileDescription) }}
			</p>
		</div>

		<div v-if="ctx.curseforgeModpackFile.value" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				<IntlFormatted :message-id="messages.serverPackLabel">
					<template #optional="{ children }">
						<span class="text-primary"><component :is="() => children" /></span>
					</template>
				</IntlFormatted>
			</span>
			<ArchiveInput v-model="ctx.curseforgeServerPackFile.value" />
			<p class="m-0 leading-6 text-primary">
				{{ formatMessage(messages.serverPackDescription) }}
			</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import { LinkIcon } from '@modrinth/assets'
import { computed, useId, watch } from 'vue'

import Input from '#ui/components/base/inputs/Input.vue'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import { useVIntl } from '#ui/composables/i18n'

import { injectCreationFlowContext } from '../../creation-flow-context'
import { curseforgeMessages as messages, isCurseForgeModpackUrl } from '../../curseforge'
import ArchiveInput from './archive-input.vue'

const ctx = injectCreationFlowContext()
const { formatMessage } = useVIntl()
const linkId = useId()
const invalidUrl = computed(
	() => !!ctx.curseforgeUrl.value.trim() && !isCurseForgeModpackUrl(ctx.curseforgeUrl.value),
)

watch(ctx.curseforgeModpackFile, (file) => {
	ctx.curseforgeServerPackFile.value = null
	if (file) ctx.curseforgeUrl.value = ''
})
</script>
