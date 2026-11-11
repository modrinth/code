<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { edit } from '@/helpers/instance'
import { appSettingsQueryOptions } from '@/helpers/settings.ts'

import DefaultSettingsIntro from './default-settings-intro.vue'
import { injectInstanceSettings } from './instance-settings-context'
import ResetToDefaultButton from './reset-to-default-button.vue'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'

type HookKey = 'pre_launch' | 'wrapper' | 'post_exit'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()

const hooks = useInstanceSettingsDraft<Record<HookKey, string | null>>(
	() => ({
		pre_launch: instance.value.hooks.pre_launch || null,
		wrapper: instance.value.hooks.wrapper || null,
		post_exit: instance.value.hooks.post_exit || null,
	}),
	(value) =>
		edit(instance.value.id, {
			hooks: {
				pre_launch: value.pre_launch ?? '',
				wrapper: value.wrapper ?? '',
				post_exit: value.post_exit ?? '',
			},
		}),
)

const settingsQuery = useQuery(appSettingsQueryOptions())
await settingsQuery.suspense().catch(handleError)

const defaultHooks = computed<Record<HookKey, string>>(() => ({
	pre_launch: settingsQuery.data.value?.hooks.pre_launch ?? '',
	wrapper: settingsQuery.data.value?.hooks.wrapper ?? '',
	post_exit: settingsQuery.data.value?.hooks.post_exit ?? '',
}))

const messages = defineMessages({
	defaultSettingsDescription: {
		id: 'instance.settings.tabs.hooks.default-settings-description',
		defaultMessage: 'Your instance uses the default launch hooks unless you override them here.',
	},
	hookVariablesDescription: {
		id: 'instance.settings.tabs.hooks.variables.description',
		defaultMessage:
			'Hooks run in the working directory of the instance, with the following variables:',
	},
	instanceNameDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-name.description',
		defaultMessage: '$INST_NAME: The name of the instance',
	},
	instanceIdDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-id.description',
		defaultMessage: "$INST_ID: The name of the instance's folder",
	},
	instanceDirDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-dir.description',
		defaultMessage: "$INST_DIR: The absolute path to the instance's folder",
	},
	instanceMcDirDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-mc-dir.description',
		defaultMessage: '$INST_MC_DIR: An alias for $INST_DIR',
	},
	instanceJavaDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-java.description',
		defaultMessage: '$INST_JAVA: The absolute path to the java binary',
	},
	instanceJavaArgsDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-java-args.description',
		defaultMessage: '$INST_JAVA_ARGS: The JVM Arguments provided to the game',
	},
	preLaunch: {
		id: 'instance.settings.tabs.hooks.pre-launch',
		defaultMessage: 'Pre-launch',
	},
	preLaunchDescription: {
		id: 'instance.settings.tabs.hooks.pre-launch.description',
		defaultMessage: 'Ran before the instance is launched.',
	},
	preLaunchEnter: {
		id: 'instance.settings.tabs.hooks.pre-launch.enter',
		defaultMessage: 'Enter pre-launch command...',
	},
	wrapper: {
		id: 'instance.settings.tabs.hooks.wrapper',
		defaultMessage: 'Wrapper',
	},
	wrapperDescription: {
		id: 'instance.settings.tabs.hooks.wrapper.description',
		defaultMessage: 'Wrapper command for launching Minecraft.',
	},
	wrapperEnter: {
		id: 'instance.settings.tabs.hooks.wrapper.enter',
		defaultMessage: 'Enter wrapper command...',
	},
	postExit: {
		id: 'instance.settings.tabs.hooks.post-exit',
		defaultMessage: 'Post-exit',
	},
	postExitDescription: {
		id: 'instance.settings.tabs.hooks.post-exit.description',
		defaultMessage: 'Ran after the game closes.',
	},
	postExitEnter: {
		id: 'instance.settings.tabs.hooks.post-exit.enter',
		defaultMessage: 'Enter post-exit command...',
	},
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<DefaultSettingsIntro
			tab="hooks"
			:description="formatMessage(messages.defaultSettingsDescription)"
		/>

		<section class="flex flex-col gap-2.5">
			<div class="flex items-center justify-between gap-4">
				<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.preLaunch) }}
				</h2>
				<ResetToDefaultButton
					:disabled="hooks.pre_launch === null"
					:setting="formatMessage(messages.preLaunch)"
					@reset="hooks.pre_launch = null"
				/>
			</div>
			<Input
				id="pre-launch"
				:model-value="hooks.pre_launch ?? defaultHooks.pre_launch"
				autocomplete="off"
				:placeholder="formatMessage(messages.preLaunchEnter)"
				wrapper-class="w-full"
				@update:model-value="(value) => (hooks.pre_launch = String(value ?? ''))"
			/>
			<p class="m-0">
				{{ formatMessage(messages.preLaunchDescription) }}
			</p>
		</section>

		<section class="flex flex-col gap-2.5">
			<div class="flex items-center justify-between gap-4">
				<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.wrapper) }}
				</h2>
				<ResetToDefaultButton
					:disabled="hooks.wrapper === null"
					:setting="formatMessage(messages.wrapper)"
					@reset="hooks.wrapper = null"
				/>
			</div>
			<Input
				id="wrapper"
				:model-value="hooks.wrapper ?? defaultHooks.wrapper"
				autocomplete="off"
				:placeholder="formatMessage(messages.wrapperEnter)"
				wrapper-class="w-full"
				@update:model-value="(value) => (hooks.wrapper = String(value ?? ''))"
			/>
			<p class="m-0">
				{{ formatMessage(messages.wrapperDescription) }}
			</p>
		</section>

		<section class="flex flex-col gap-2.5">
			<div class="flex items-center justify-between gap-4">
				<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.postExit) }}
				</h2>
				<ResetToDefaultButton
					:disabled="hooks.post_exit === null"
					:setting="formatMessage(messages.postExit)"
					@reset="hooks.post_exit = null"
				/>
			</div>
			<Input
				id="post-exit"
				:model-value="hooks.post_exit ?? defaultHooks.post_exit"
				autocomplete="off"
				:placeholder="formatMessage(messages.postExitEnter)"
				wrapper-class="w-full"
				@update:model-value="(value) => (hooks.post_exit = String(value ?? ''))"
			/>
			<p class="m-0">
				{{ formatMessage(messages.postExitDescription) }}
			</p>
		</section>

		<div>
			<div class="m-0">
				{{ formatMessage(messages.hookVariablesDescription) }}
			</div>
			<ul class="m-0 mt-2">
				<li>{{ formatMessage(messages.instanceNameDescription) }}</li>
				<li>{{ formatMessage(messages.instanceIdDescription) }}</li>
				<li>{{ formatMessage(messages.instanceDirDescription) }}</li>
				<li>{{ formatMessage(messages.instanceMcDirDescription) }}</li>
				<li>{{ formatMessage(messages.instanceJavaDescription) }}</li>
				<li>{{ formatMessage(messages.instanceJavaArgsDescription) }}</li>
			</ul>
		</div>
	</div>
</template>
