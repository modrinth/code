<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { appSettingsQueryOptions } from '@/helpers/settings.ts'

import SettingsSourceHeader from './settings-source-header.vue'
import { useSettingsSource } from './use-settings-source'

type HookKey = 'pre_launch' | 'wrapper' | 'post_exit'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const settingsQuery = useQuery(appSettingsQueryOptions())
await settingsQuery.suspense().catch(handleError)

const defaultHooks = computed<Record<HookKey, string>>(() => ({
	pre_launch: settingsQuery.data.value?.hooks.pre_launch ?? '',
	wrapper: settingsQuery.data.value?.hooks.wrapper ?? '',
	post_exit: settingsQuery.data.value?.hooks.post_exit ?? '',
}))

const {
	source,
	values: hooks,
	update,
	debouncedField,
} = useSettingsSource<Record<HookKey, string | null>>({
	read: (current) => ({
		pre_launch: current.hooks.pre_launch || null,
		wrapper: current.hooks.wrapper || null,
		post_exit: current.hooks.post_exit || null,
	}),
	write: (value) => ({
		hooks: {
			pre_launch: value.pre_launch ?? '',
			wrapper: value.wrapper ?? '',
			post_exit: value.post_exit ?? '',
		},
	}),
	defaults: () => defaultHooks.value,
})

function hookField(key: HookKey) {
	return debouncedField(
		() => hooks.value[key] ?? defaultHooks.value[key],
		(value) => update({ [key]: value }),
	)
}

const preLaunch = hookField('pre_launch')
const wrapper = hookField('wrapper')
const postExit = hookField('post_exit')

const messages = defineMessages({
	settingsHeading: {
		id: 'instance.settings.tabs.hooks.settings-heading',
		defaultMessage: 'Launch hooks settings',
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
		<SettingsSourceHeader
			v-model="source"
			tab="hooks"
			:title="formatMessage(messages.settingsHeading)"
		/>

		<div
			class="flex flex-col gap-6 transition-opacity"
			:class="{ 'opacity-60': source === 'app' }"
			:inert="source === 'app'"
		>
			<section class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.preLaunch) }}
				</h2>
				<Input
					id="pre-launch"
					:model-value="preLaunch.model"
					autocomplete="off"
					:placeholder="formatMessage(messages.preLaunchEnter)"
					wrapper-class="w-full"
					@focus="preLaunch.onFocus"
					@blur="preLaunch.onBlur"
					@update:model-value="(value) => (preLaunch.model = String(value ?? ''))"
				/>
				<p class="m-0">
					{{ formatMessage(messages.preLaunchDescription) }}
				</p>
			</section>

			<section class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.wrapper) }}
				</h2>
				<Input
					id="wrapper"
					:model-value="wrapper.model"
					autocomplete="off"
					:placeholder="formatMessage(messages.wrapperEnter)"
					wrapper-class="w-full"
					@focus="wrapper.onFocus"
					@blur="wrapper.onBlur"
					@update:model-value="(value) => (wrapper.model = String(value ?? ''))"
				/>
				<p class="m-0">
					{{ formatMessage(messages.wrapperDescription) }}
				</p>
			</section>

			<section class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.postExit) }}
				</h2>
				<Input
					id="post-exit"
					:model-value="postExit.model"
					autocomplete="off"
					:placeholder="formatMessage(messages.postExitEnter)"
					wrapper-class="w-full"
					@focus="postExit.onFocus"
					@blur="postExit.onBlur"
					@update:model-value="(value) => (postExit.model = String(value ?? ''))"
				/>
				<p class="m-0">
					{{ formatMessage(messages.postExitDescription) }}
				</p>
			</section>
		</div>

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
