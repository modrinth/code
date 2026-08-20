<script setup lang="ts">
import { BanIcon, SpinnerIcon, TagCategoryWandSparklesIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { nextTick, ref, useTemplateRef } from 'vue'

import IconEditorModal from '@/components/ui/instance_settings/icon-editor-modal/index.vue'
import { toError } from '@/helpers/errors'
import { list as listInstances } from '@/helpers/instance'

import icon01 from './assets/01.png'
import icon02 from './assets/02.png'
import icon03 from './assets/03.png'
import icon04 from './assets/04.png'
import icon05 from './assets/05.png'
import icon06 from './assets/06.png'
import icon07 from './assets/07.png'
import icon08 from './assets/08.png'
import icon09 from './assets/09.png'

const emit = defineEmits<{
	close: []
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const iconEditorModal = useTemplateRef<InstanceType<typeof IconEditorModal>>('iconEditorModal')
const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()
const iconlessInstanceIds = ref<string[]>([])
const loading = ref(false)
const applying = ref(false)
const icons = [icon01, icon02, icon03, icon04, icon05, icon06, icon07, icon08, icon09]

const messages = defineMessages({
	badge: {
		id: 'app.icon-editor.apply-icons-modal.badge',
		defaultMessage: 'New this update',
	},
	title: {
		id: 'app.icon-editor.apply-icons-modal.title',
		defaultMessage: 'Custom icon editor',
	},
	description: {
		id: 'app.icon-editor.apply-icons-modal.description',
		defaultMessage:
			'Create custom icons for your instances right in the Modrinth App. Mix and match backgrounds with symbols from Minecraft and popular mods!',
	},
	instancesWithoutIcons: {
		id: 'app.icon-editor.apply-icons-modal.instances-without-icons',
		defaultMessage:
			'{count, plural, one {# instance doesn’t} other {# instances don’t}} have icons yet. Want us to give {count, plural, one {it} other {them}} random ones to get started?',
	},
	skip: {
		id: 'app.icon-editor.apply-icons-modal.skip',
		defaultMessage: 'Skip',
	},
	randomIcons: {
		id: 'app.icon-editor.apply-icons-modal.random-icons',
		defaultMessage: 'Random icons',
	},
	close: {
		id: 'app.icon-editor.apply-icons-modal.close',
		defaultMessage: 'Close',
	},
	applyError: {
		id: 'app.icon-editor.apply-icons-modal.apply-error',
		defaultMessage: 'Failed to apply random icons to instances.',
	},
	applySuccess: {
		id: 'app.icon-editor.apply-icons-modal.apply-success',
		defaultMessage: 'Randomized icon for {num, plural, one {# instance} other {# instances}}.',
	},
})

async function show() {
	modal.value?.show()
	loading.value = true
	try {
		iconlessInstanceIds.value = (await listInstances())
			.filter((instance) => !instance.icon_path)
			.map((instance) => instance.id)
	} catch (error) {
		handleError(toError(error))
		iconlessInstanceIds.value = []
	} finally {
		loading.value = false
	}
}

function hide() {
	modal.value?.hide()
}

function handleHide() {
	emit('close')
}

async function applyIcons() {
	if (applying.value || loading.value) return

	applying.value = true
	const instanceCount = iconlessInstanceIds.value.length
	let applied = false
	try {
		const results = await Promise.all(
			iconlessInstanceIds.value.map(async (instanceId) => {
				const generated = await iconEditorModal.value?.randomize()
				if (!generated) return false

				return (
					(await iconEditorModal.value?.applyGeneratedIcon(instanceId, generated.config)) ?? false
				)
			}),
		)
		applied = results.every(Boolean)
	} finally {
		applying.value = false
	}

	if (applied) {
		await nextTick()
		hide()
		addNotification({
			type: 'success',
			title: formatMessage(messages.applySuccess, { num: instanceCount }),
		})
	} else {
		addNotification({
			type: 'error',
			title: formatMessage(messages.applyError),
		})
	}
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		hide-header
		no-padding
		width="770px"
		max-width="calc(100vw - 2rem)"
		:aria-label="formatMessage(messages.title)"
		:on-after-hide="handleHide"
		:disable-close="applying"
		class="!overflow-hidden !rounded-[20px]"
	>
		<div class="grid w-[768px] max-w-full grid-cols-2">
			<section class="flex min-w-0 flex-col gap-6 bg-surface-3 p-8">
				<div
					class="flex h-8 w-fit items-center rounded-full border border-solid border-brand bg-brand-highlight px-2.5 text-sm font-base leading-5 text-brand"
				>
					{{ formatMessage(messages.badge) }}
				</div>

				<div class="flex min-w-0 flex-col gap-4">
					<h2 class="m-0 text-2xl font-semibold leading-6 text-contrast">
						{{ formatMessage(messages.title) }}
					</h2>
					<p class="m-0 leading-6 text-primary">
						{{ formatMessage(messages.description) }}
					</p>
					<p v-if="iconlessInstanceIds.length" class="m-0 leading-6 text-primary">
						{{
							formatMessage(messages.instancesWithoutIcons, {
								count: iconlessInstanceIds.length,
							})
						}}
					</p>
				</div>

				<div class="flex mt-auto items-center gap-2.5">
					<Button size="lg" :disabled="applying" @click="hide">
						<template v-if="iconlessInstanceIds.length === 0">
							{{ formatMessage(messages.close) }}
						</template>
						<template v-else>
							<BanIcon />
							{{ formatMessage(messages.skip) }}
						</template>
					</Button>
					<Button
						v-if="iconlessInstanceIds.length !== 0"
						type="colored"
						color="brand"
						size="lg"
						:disabled="loading || applying"
						@click="applyIcons"
					>
						<SpinnerIcon v-if="applying" class="animate-spin" />
						<TagCategoryWandSparklesIcon v-else />
						{{ formatMessage(messages.randomIcons) }}
					</Button>
				</div>
			</section>

			<section
				class="relative flex min-w-0 items-center justify-center border-0 border-l border-solid border-surface-5 bg-surface-2 p-10"
			>
				<IconButton
					type="quiet"
					size="sm"
					:label="formatMessage(messages.close)"
					class="!absolute right-6 top-6 z-10"
					:disabled="applying"
					@click="hide"
				>
					<XIcon />
				</IconButton>

				<div class="grid size-60 grid-cols-3 grid-rows-3 gap-4" aria-hidden="true">
					<Avatar
						v-for="(icon, index) in icons"
						:key="index"
						:src="icon"
						size="100%"
						class="min-h-0 min-w-0 !rounded-2xl"
					/>
				</div>
			</section>
		</div>
	</NewModal>
	<IconEditorModal ref="iconEditorModal" />
</template>
