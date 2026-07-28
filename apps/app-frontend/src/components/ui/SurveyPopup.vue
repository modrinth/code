<script setup lang="ts">
import { NotepadTextIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { type } from '@tauri-apps/plugin-os'
import { $fetch } from 'ofetch'
import { onMounted, ref } from 'vue'

import { hide_ads_window, show_ads_window } from '@/helpers/ads.js'
import { list } from '@/helpers/instance'
import { get as getCreds } from '@/helpers/mr_auth.ts'

type Survey = {
	id: string
	tally_id: string
	type: string
	condition?: string
	assigned_users?: string[]
	dismissed_users?: string[]
}

type TallyApi = {
	openPopup: (formId: string, options: object) => void
}

const tallyWindow = window as Window & { Tally?: TallyApi }

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const availableSurvey = ref<Survey | null>(null)

const messages = defineMessages({
	surveyTitle: {
		id: 'app.survey.title',
		defaultMessage: 'Hey there Modrinth user!',
	},
	surveyBody: {
		id: 'app.survey.body',
		defaultMessage:
			'Would you mind answering a few questions about your experience with Modrinth App?',
	},
	surveyFooter: {
		id: 'app.survey.footer',
		defaultMessage:
			'This feedback will go directly to the Modrinth team and help guide future updates!',
	},
	takeSurvey: {
		id: 'app.survey.take-survey',
		defaultMessage: 'Take survey',
	},
	surveyNoThanks: {
		id: 'app.survey.no-thanks',
		defaultMessage: 'No thanks',
	},
})

function cleanupOldSurveyDisplayData() {
	const threeWeeksAgo = new Date()
	threeWeeksAgo.setDate(threeWeeksAgo.getDate() - 21)

	for (let i = 0; i < localStorage.length; i++) {
		const key = localStorage.key(i)

		if (key?.startsWith('survey-') && key.endsWith('-display')) {
			const dateValue = new Date(localStorage.getItem(key) ?? '')
			if (dateValue < threeWeeksAgo) {
				localStorage.removeItem(key)
			}
		}
	}
}

async function openSurvey() {
	if (!availableSurvey.value) {
		console.error('No survey to open')
		return
	}

	const creds = await getCreds().catch(handleError)
	const userId = creds?.user_id

	const formId = availableSurvey.value.tally_id

	const popupOptions = {
		layout: 'modal',
		width: 700,
		autoClose: 2000,
		hideTitle: true,
		hiddenFields: {
			user_id: userId,
		},
		onOpen: () => console.info('Opened user survey'),
		onClose: () => {
			console.info('Closed user survey')
			show_ads_window()
		},
		onSubmit: () => console.info('Active user survey submitted'),
	}

	try {
		hide_ads_window()
		if (tallyWindow.Tally?.openPopup) {
			console.info(`Opening Tally popup for user survey (form ID: ${formId})`)
			dismissSurvey()
			tallyWindow.Tally.openPopup(formId, popupOptions)
		} else {
			console.warn('Tally script not yet loaded')
			show_ads_window()
		}
	} catch (e) {
		console.error('Error opening Tally popup:', e)
		show_ads_window()
	}

	console.info(`Found user survey to show with tally_id: ${formId}`)
	tallyWindow.Tally?.openPopup(formId, popupOptions)
}

function dismissSurvey() {
	if (!availableSurvey.value) return
	localStorage.setItem(`survey-${availableSurvey.value.id}-display`, String(new Date()))
	availableSurvey.value = null
}

async function processPendingSurveys() {
	function isWithinLastTwoWeeks(date: string | Date | null | undefined) {
		if (!date) return false
		const twoWeeksAgo = new Date()
		twoWeeksAgo.setDate(twoWeeksAgo.getDate() - 14)
		return new Date(date) >= twoWeeksAgo
	}

	cleanupOldSurveyDisplayData()

	const creds = await getCreds().catch(handleError)
	const userId = creds?.user_id

	const instances = (await list().catch(handleError)) ?? []
	const isActivePlayer = instances.some(
		(instance) =>
			isWithinLastTwoWeeks(instance.last_played) && !isWithinLastTwoWeeks(instance.created),
	)

	let surveys: Survey[] = []
	try {
		surveys = await $fetch('https://api.modrinth.com/v2/surveys')
	} catch (e) {
		console.error('Error fetching surveys:', e)
	}

	const surveyToShow = surveys.find(
		(survey) =>
			!!(
				localStorage.getItem(`survey-${survey.id}-display`) === null &&
				survey.type === 'tally_app' &&
				((survey.condition === 'active_player' && isActivePlayer) ||
					(!!userId &&
						survey.assigned_users?.includes(userId) &&
						!survey.dismissed_users?.includes(userId)))
			),
	)

	if (surveyToShow) {
		availableSurvey.value = surveyToShow
	} else {
		console.info('No user survey to show')
	}
}

onMounted(async () => {
	const osType = await type()
	if (osType === 'windows') {
		await processPendingSurveys()
	} else {
		console.info('Skipping user surveys on non-Windows platforms')
	}
})
</script>

<template>
	<transition name="popup-survey">
		<div
			v-if="availableSurvey"
			class="w-[400px] z-20 fixed -bottom-12 pb-16 right-[--right-bar-width] mr-4 rounded-t-2xl card-shadow bg-bg-raised border-surface-5 border-[1px] border-solid border-b-0 p-4"
		>
			<h2 class="text-lg font-extrabold mt-0 mb-2">
				{{ formatMessage(messages.surveyTitle) }}
			</h2>
			<p class="m-0 leading-tight">
				{{ formatMessage(messages.surveyBody) }}
			</p>
			<p class="mt-3 mb-4 leading-tight">
				{{ formatMessage(messages.surveyFooter) }}
			</p>
			<div class="flex gap-2">
				<ButtonStyled color="brand">
					<button @click="openSurvey">
						<NotepadTextIcon />
						{{ formatMessage(messages.takeSurvey) }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="dismissSurvey">
						<XIcon />
						{{ formatMessage(messages.surveyNoThanks) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</transition>
</template>

<style scoped>
.popup-survey-enter-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s cubic-bezier(0.51, 1.08, 0.35, 1.15);
	transform-origin: top center;
}

.popup-survey-leave-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s cubic-bezier(0.68, -0.17, 0.23, 0.11);
	transform-origin: top center;
}

.popup-survey-enter-from,
.popup-survey-leave-to {
	opacity: 0;
	transform: translateY(10rem) scale(0.8) scaleY(1.6);
}
</style>
