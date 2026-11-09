<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, useVIntl } from '@modrinth/ui'

import { edit } from '@/helpers/instance'
import { get } from '@/helpers/settings.ts'
import type { AppSettings } from '@/helpers/types'

import DefaultSettingsIntro from './default-settings-intro.vue'
import { injectInstanceSettings } from './instance-settings-context'
import ResetToDefaultButton from './reset-to-default-button.vue'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()

const globalSettings = (await get().catch(handleError)) as AppSettings

type HookKey = 'pre_launch' | 'wrapper' | 'post_exit'

const defaultHooks: Record<HookKey, string> = {
	pre_launch: globalSettings.hooks.pre_launch ?? '',
	wrapper: globalSettings.hooks.wrapper ?? '',
	post_exit: globalSettings.hooks.post_exit ?? '',
}
const { draft: hooks } = useInstanceSettingsDraft<Record<HookKey, string>>(
	{
		pre_launch: instance.value.hooks.pre_launch || defaultHooks.pre_launch,
		wrapper: instance.value.hooks.wrapper || defaultHooks.wrapper,
		post_exit: instance.value.hooks.post_exit || defaultHooks.post_exit,
	},
	(value) => {
		const override = (key: HookKey) => (value[key] !== defaultHooks[key] ? value[key] : '')
		return edit(instance.value.id, {
			hooks: {
				pre_launch: override('pre_launch'),
				wrapper: override('wrapper'),
				post_exit: override('post_exit'),
			},
		})
	},
)

function hookOverridden(key: HookKey) {
	return hooks.value[key] !== defaultHooks[key]
}

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
					:disabled="!hookOverridden('pre_launch')"
					:setting="formatMessage(messages.preLaunch)"
					@reset="hooks.pre_launch = defaultHooks.pre_launch"
				/>
			</div>
			<Input
				id="pre-launch"
				v-model="hooks.pre_launch"
				autocomplete="off"
				:placeholder="formatMessage(messages.preLaunchEnter)"
				wrapper-class="w-full"
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
					:disabled="!hookOverridden('wrapper')"
					:setting="formatMessage(messages.wrapper)"
					@reset="hooks.wrapper = defaultHooks.wrapper"
				/>
			</div>
			<Input
				id="wrapper"
				v-model="hooks.wrapper"
				autocomplete="off"
				:placeholder="formatMessage(messages.wrapperEnter)"
				wrapper-class="w-full"
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
					:disabled="!hookOverridden('post_exit')"
					:setting="formatMessage(messages.postExit)"
					@reset="hooks.post_exit = defaultHooks.post_exit"
				/>
			</div>
			<Input
				id="post-exit"
				v-model="hooks.post_exit"
				autocomplete="off"
				:placeholder="formatMessage(messages.postExitEnter)"
				wrapper-class="w-full"
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
