<script setup lang="ts">
import { login as login_flow, set_default_user } from '@/helpers/auth.js'
import { handleSevereError } from '@/store/error.js'
import {
	CheckIcon,
	CopyIcon,
	DropdownIcon,
	LogInIcon,
	SupportChatIcon,
	WrenchIcon,
} from '@modrinth/assets'
import { Admonition, ButtonStyled, Collapsible, NewModal } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { type MinecraftAuthError, minecraftAuthErrors } from './minecraft-auth-errors'

const modal = ref<InstanceType<typeof NewModal>>()
const rawError = ref<string>('')
const matchedError = ref<MinecraftAuthError | null>(null)
const debugCollapsed = ref(true)
const copied = ref(false)
const loadingSignIn = ref(false)

interface minecraftAuthError {
	XErr: string
}

function show(errorVal: { message?: string }) {
	rawError.value = errorVal?.message ?? String(errorVal) ?? 'Unknown error'

	matchedError.value = minecraftAuthErrors.find((e) => rawError.value.includes(e.errorCode)) ?? null

	debugCollapsed.value = true
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})

async function signInAgain() {
	try {
		loadingSignIn.value = true
		const loggedIn = await login_flow()
		if (loggedIn) {
			await set_default_user(loggedIn.profile.id)
		}
		loadingSignIn.value = false
		modal.value?.hide()
	} catch (err) {
		loadingSignIn.value = false
		handleSevereError(err)
	}
}

const debugInfo = computed(() => rawError.value || 'No error message.')

async function copyToClipboard(text: string) {
	await navigator.clipboard.writeText(text)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 3000)
}
</script>

<template>
	<NewModal ref="modal" header="Sign in Failed" :max-width="'600px'">
		<div class="flex flex-col gap-4">
			<Admonition
				type="warning"
				body="	We couldn't sign you into your Microsoft account. This may be due to account restrictions or
				regional limitations."
			>
			</Admonition>

			<!-- Matched error details -->
			<div v-if="matchedError" class="bg-surface-2 rounded-2xl p-4 px-5">
				<h3 class="text-base font-bold m-0 mb-1">What we think happened</h3>
				<p class="text-sm text-secondary m-0 mb-4">
					{{ matchedError.whatHappened }}
				</p>

				<h3 class="text-base font-bold m-0 mb-2">How to fix it</h3>
				<ol class="list-none flex flex-col gap-2 m-0 pl-0">
					<li
						v-for="(step, index) in matchedError.stepsToFix"
						:key="index"
						class="flex items-baseline gap-3"
					>
						<span
							class="inline-flex items-center justify-center shrink-0 w-5 h-5 rounded-full bg-surface-4 border border-solid border-solid-5 text-xs font-bold"
						>
							{{ index + 1 }}
						</span>
						<!-- eslint-disable-next-line vue/no-v-html -->
						<span class="text-sm [&_a]:text-brand [&_a]:underline" v-html="step" />
					</li>
				</ol>
			</div>

			<!-- Action buttons -->
			<div class="flex items-center gap-2">
				<ButtonStyled>
					<a href="https://support.modrinth.com" @click="modal?.hide()" class="!w-full">
						<SupportChatIcon /> Contact support
					</a>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="loadingSignIn" @click="signInAgain" class="!w-full">
						<LogInIcon /> Sign in again
					</button>
				</ButtonStyled>
			</div>

			<!-- Debug info -->
			<div class="bg-button-bg rounded-xl overflow-clip">
				<button
					class="flex items-center justify-between w-full bg-transparent border-0 px-4 py-3 cursor-pointer"
					@click="debugCollapsed = !debugCollapsed"
				>
					<span class="flex items-center gap-2 text-contrast font-extrabold m-0">
						<WrenchIcon class="h-4 w-4" />
						Debug information
					</span>
					<DropdownIcon
						class="h-5 w-5 text-secondary transition-transform"
						:class="{ 'rotate-180': !debugCollapsed }"
					/>
				</button>
				<Collapsible :collapsed="debugCollapsed">
					<div class="px-4 pb-3">
						<pre class="m-0 p-3 bg-bg rounded-lg text-xs overflow-auto">{{ debugInfo }}</pre>
						<div class="mt-2 flex justify-end">
							<ButtonStyled>
								<button :disabled="copied" @click="copyToClipboard(debugInfo)">
									<template v-if="copied"> <CheckIcon class="text-green" /> Copied! </template>
									<template v-else> <CopyIcon /> Copy </template>
								</button>
							</ButtonStyled>
						</div>
					</div>
				</Collapsible>
			</div>
		</div>
	</NewModal>
</template>
