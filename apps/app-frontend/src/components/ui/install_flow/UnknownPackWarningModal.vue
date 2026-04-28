<template>
	<ModalWrapper ref="modal" :header="formatMessage(messages.header)">
		<div class="max-w-[31rem] flex flex-col gap-6">
			<Admonition
				type="warning"
				:header="formatMessage(messages.warningTitle)"
				:body="formatMessage(messages.warningBody)"
			/>
			<div>
				<p class="mt-0 leading-tight">
					{{ formatMessage(messages.body) }}
				</p>
				<p class="text-orange font-semibold mb-0 leading-tight">
					{{ formatMessage(messages.malwareStatement) }}
				</p>
			</div>
			<Checkbox v-model="dontShowAgain" :label="formatMessage(messages.dontShowAgain)" />
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button>
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button>
						<CircleArrowRightIcon />
						{{ formatMessage(messages.installAnyway) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</ModalWrapper>
</template>

<script setup>
import { CircleArrowRightIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Checkbox,
	commonMessages,
	defineMessages,
	useVIntl,
} from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

const { formatMessage } = useVIntl()

const dontShowAgain = ref(false)
const modal = useTemplateRef('modal')

const messages = defineMessages({
	header: {
		id: 'unknown-pack-warning-modal.header',
		defaultMessage: 'Confirm installation',
	},
	warningTitle: {
		id: 'unknown-pack-warning-modal.warning.title',
		defaultMessage: 'Unknown file warning',
	},
	warningBody: {
		id: 'unknown-pack-warning-modal.warning.body',
		defaultMessage: `We couldn't find this file on Modrinth. We strongly recommend only installing files from sources you trust.`,
	},
	body: {
		id: 'unknown-pack-warning-modal.body',
		defaultMessage: `A file is only reviewed if it’s uploaded to Modrinth, regardless of its file format (including .mrpack).`,
	},
	malwareStatement: {
		id: 'unknown-pack-warning-modal.malware-statement',
		defaultMessage: `Malware is often distributed through modpack files by sharing them on platforms like Discord.`,
	},
	dontShowAgain: {
		id: 'unknown-pack-warning-modal.dont-show-again',
		defaultMessage: `Don't show this warning again`,
	},
	installAnyway: {
		id: 'unknown-pack-warning-modal.install-anyway',
		defaultMessage: `Install anyway`,
	},
})

function show() {
	modal.value.show()
}

defineExpose({ show })
</script>
