<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, Input, useVIntl } from '@modrinth/ui'

import { useDefaultInstanceSettings } from '@/composables/use-default-instance-settings'

const { formatMessage } = useVIntl()
const { settings, settingsQuery } = useDefaultInstanceSettings(
	(value) => ({ hooks: { ...value.hooks } }),
	(value) => ({ hooks: { ...value.hooks } }),
)

const messages = defineMessages({
	preLaunchHookTitle: {
		id: 'app.settings.default-instance-options.pre-launch-hook.title',
		defaultMessage: 'Pre-launch hook',
	},
	preLaunchHookPlaceholder: {
		id: 'app.settings.default-instance-options.pre-launch-hook.placeholder',
		defaultMessage: 'Enter pre-launch command...',
	},
	preLaunchHookDescription: {
		id: 'app.settings.default-instance-options.pre-launch-hook.description',
		defaultMessage: 'Runs before the instance starts.',
	},
	wrapperHookTitle: {
		id: 'app.settings.default-instance-options.wrapper-hook.title',
		defaultMessage: 'Wrapper hook',
	},
	wrapperHookPlaceholder: {
		id: 'app.settings.default-instance-options.wrapper-hook.placeholder',
		defaultMessage: 'Enter wrapper command...',
	},
	wrapperHookDescription: {
		id: 'app.settings.default-instance-options.wrapper-hook.description',
		defaultMessage: 'Command used to wrap the Minecraft launch process.',
	},
	postExitHookTitle: {
		id: 'app.settings.default-instance-options.post-exit-hook.title',
		defaultMessage: 'Post-exit hook',
	},
	postExitHookPlaceholder: {
		id: 'app.settings.default-instance-options.post-exit-hook.placeholder',
		defaultMessage: 'Enter post-exit command...',
	},
	postExitHookDescription: {
		id: 'app.settings.default-instance-options.post-exit-hook.description',
		defaultMessage: 'Runs after the game closes.',
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
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<template v-if="settings">
			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.preLaunchHookTitle) }}
				</h2>
				<Input
					id="pre-launch"
					v-model="settings.hooks.pre_launch"
					:aria-label="formatMessage(messages.preLaunchHookTitle)"
					autocomplete="off"
					type="text"
					:placeholder="formatMessage(messages.preLaunchHookPlaceholder)"
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.preLaunchHookDescription) }}
				</p>
			</div>
			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.wrapperHookTitle) }}
				</h2>
				<Input
					id="wrapper"
					v-model="settings.hooks.wrapper"
					:aria-label="formatMessage(messages.wrapperHookTitle)"
					autocomplete="off"
					type="text"
					:placeholder="formatMessage(messages.wrapperHookPlaceholder)"
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.wrapperHookDescription) }}
				</p>
			</div>
			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.postExitHookTitle) }}
				</h2>
				<Input
					id="post-exit"
					v-model="settings.hooks.post_exit"
					:aria-label="formatMessage(messages.postExitHookTitle)"
					autocomplete="off"
					type="text"
					:placeholder="formatMessage(messages.postExitHookPlaceholder)"
				wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.postExitHookDescription) }}
				</p>
			</div>
			<div class="m-0 leading-tight">
				{{ formatMessage(messages.hookVariablesDescription) }}
				<ul>
					<li>{{ formatMessage(messages.instanceNameDescription) }}</li>
					<li>{{ formatMessage(messages.instanceIdDescription) }}</li>
					<li>{{ formatMessage(messages.instanceDirDescription) }}</li>
					<li>{{ formatMessage(messages.instanceMcDirDescription) }}</li>
					<li>{{ formatMessage(messages.instanceJavaDescription) }}</li>
					<li>{{ formatMessage(messages.instanceJavaArgsDescription) }}</li>
				</ul>
			</div>
		</template>
		<Button v-else-if="settingsQuery.isError.value" @click="settingsQuery.refetch()">
			{{ formatMessage(commonMessages.refreshButton) }}
		</Button>
		<div v-else class="flex items-center gap-2 text-secondary">
			<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
			{{ formatMessage(commonMessages.loadingLabel) }}
		</div>
	</div>
</template>
