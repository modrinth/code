<template>
	<CreatorWithdrawModal
		ref="withdrawModal"
		:balance="userBalance"
		:preloaded-payment-data="preloadedPaymentMethods"
		@refresh-data="refreshData"
	/>
	<div class="mb-20 flex flex-col gap-6 lg:pl-8">
		<div class="flex flex-col gap-4 md:gap-5">
			<div class="flex flex-col gap-1">
				<span class="text-xl font-semibold text-contrast md:text-2xl">{{
					formatMessage(messages.balanceLabel)
				}}</span>
				<span
					class="bg-gradient-to-r from-brand-purple via-brand-orange via-20% to-brand-orange bg-clip-text text-3xl font-extrabold text-transparent md:text-4xl"
					>{{ formatMoney(grandTotal) }}</span
				>
			</div>

			<div class="flex h-3 w-full gap-2 overflow-hidden rounded-full bg-bg-raised md:h-4">
				<div
					v-for="(seg, index) in segments"
					:key="seg.key"
					class="h-full hover:brightness-105"
					:style="{ width: seg.widthPct }"
					@mouseenter="hoveredSeg = seg.key"
					@mouseleave="hoveredSeg = null"
				>
					<span
						class="block h-full w-full transition duration-150"
						:class="[
							seg.class,
							seg.key === 'available' ? 'gradient-border' : '',
							index === 0 ? 'rounded-l-full' : '',
							index === segments.length - 1 ? 'rounded-r-full' : '',
						]"
					></span>
				</div>
			</div>

			<div class="flex flex-col">
				<div
					class="flex flex-row justify-between border-0 !border-b-[2px] border-solid border-divider p-1.5 md:p-2"
				>
					<span class="my-auto flex flex-row items-center gap-2 text-sm leading-none md:text-base"
						><span
							class="gradient-border my-auto block size-4 rounded-full bg-brand-green md:size-5"
						></span>
						{{ formatMessage(messages.availableNow) }}</span
					>
					<span
						class="text-base font-semibold text-contrast md:text-lg"
						:class="{ '!text-green': hoveredSeg === 'available' }"
						>{{ formatMoney(totalAvailable) }}</span
					>
				</div>

				<div
					v-for="(date, i) in dateSegments"
					:key="date.date"
					class="flex flex-row justify-between border-0 !border-b-[2px] border-solid border-divider p-1.5 md:p-2"
				>
					<span class="my-auto flex flex-row items-center gap-2 text-sm leading-none md:text-base">
						<span
							class="zone--striped-small my-auto block size-4 rounded-full md:size-5"
							:class="[date.stripeClass, date.highlightClass]"
						></span>
						{{
							formatMessage(messages.estimatedWithDate, {
								date: date.date ? dayjs(date.date).format('MMM D, YYYY') : '',
							})
						}}
						<Tooltip theme="dismissable-prompt" :triggers="['hover', 'focus']" no-auto-focus>
							<nuxt-link
								class="inline-flex items-center justify-center text-link"
								to="/legal/cmp-info#pending"
							>
								<UnknownIcon class="inline-block size-4 align-middle md:size-5" />
							</nuxt-link>
							<template #popper>
								<div class="w-[250px] font-semibold text-contrast">
									{{ formatMessage(messages.estimatedTooltip1) }}
									<br /><br />
									{{ formatMessage(messages.estimatedTooltip2) }}
								</div>
							</template>
						</Tooltip>
					</span>
					<span
						class="text-base font-semibold text-contrast md:text-lg"
						:class="{ [date.textClass]: hoveredSeg === `upcoming-${date.date}-${i}` }"
						>{{ formatMoney(date?.amount ?? 0) }}</span
					>
				</div>

				<div class="flex flex-row justify-between p-1.5 md:p-2">
					<span class="my-auto flex flex-row items-center gap-2 text-sm leading-none md:text-base">
						<span
							class="zone--striped-small zone--striped--gray my-auto block size-4 rounded-full bg-button-bg opacity-90 md:size-5"
						></span>
						{{ formatMessage(messages.processing) }}
						<Tooltip theme="dismissable-prompt" :triggers="['hover', 'focus']" no-auto-focus>
							<InProgressIcon class="inline-block size-4 align-middle md:size-5" />
							<template #popper>
								<div class="w-[250px] font-semibold text-contrast">
									{{ formatMessage(messages.processingTooltip) }}
								</div>
							</template>
						</Tooltip>
					</span>
					<span
						class="text-base font-semibold text-contrast md:text-lg"
						:class="{ '!text-gray': hoveredSeg === 'processing' }"
						>{{ formatMoney(processingDate?.amount ?? 0) }}</span
					>
				</div>
			</div>
		</div>

		<div class="flex flex-col gap-3 md:gap-4">
			<span class="text-xl font-semibold text-contrast md:text-2xl">{{
				formatMessage(messages.withdrawHeader)
			}}</span>
			<div class="grid grid-cols-1 gap-x-4 gap-y-2 md:grid-cols-2 lg:grid-cols-3">
				<button
					class="relative flex flex-col overflow-hidden rounded-2xl bg-gradient-to-r from-green to-green-700 p-4 text-inverted shadow-md transition-all duration-200 hover:brightness-105 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:brightness-100 md:p-5"
					:disabled="hasTinMismatch"
					@click="openWithdrawModal"
				>
					<div class="relative z-10 flex flex-row justify-between">
						<span class="text-base font-semibold md:text-lg">{{
							formatMessage(messages.withdrawCardTitle)
						}}</span>
						<ArrowUpRightIcon class="my-auto size-5 md:size-6" />
					</div>
					<div class="relative z-10 text-left text-sm font-medium">
						{{ formatMessage(messages.withdrawCardDescription) }}
					</div>
					<svg
						aria-hidden="true"
						class="pointer-events-none absolute bottom-0 right-0 z-0 h-full w-auto"
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 266 100"
						fill="none"
					>
						<path
							fill-rule="evenodd"
							clip-rule="evenodd"
							d="M319.052 54.2233C319.058 37.6952 315.689 21.3441 309.156 6.19864C302.624 -8.94682 293.07 -22.559 281.094 -33.7816C269.119 -45.0042 254.982 -53.5944 239.573 -59.012C224.164 -64.4295 207.815 -66.5571 191.556 -65.2609C175.297 -63.9648 159.479 -59.2729 145.097 -51.4805C130.715 -43.688 118.08 -32.9636 107.987 -19.9818C97.8942 -6.99995 90.5617 7.95837 86.4509 23.9523C82.3401 39.9462 81.5398 56.6297 84.1004 72.9533L103.415 67.7101C100.452 45.7823 104.805 23.4811 115.783 4.35031C126.761 -14.7805 143.734 -29.6435 164.005 -37.8768C184.275 -46.1102 206.681 -47.2415 227.661 -41.0911C248.641 -34.9407 266.991 -21.8613 279.797 -3.93146L262.376 6.25239C255.476 -2.83248 246.698 -10.2779 236.659 -15.5617C226.619 -20.8455 215.561 -23.8398 204.26 -24.3345L206.032 -3.60929C217.266 -2.58081 227.949 1.79213 236.737 8.95915C245.524 16.1262 252.024 25.767 255.418 36.6684C258.812 47.5697 258.949 59.2444 255.81 70.223C252.672 81.2017 246.398 90.9937 237.78 98.3668L248.048 116.384C261.575 105.867 271.303 91.124 275.725 74.437C280.146 57.7501 279.016 40.0505 272.507 24.079L289.873 13.9453C295.192 26.0533 298.028 39.1299 298.209 52.3816L319.052 54.2233Z"
							fill="#1BD96A"
							fill-opacity="0.16"
						/>
						<path
							d="M145.331 -51.0364C159.653 -58.796 175.404 -63.4691 191.595 -64.7598C207.786 -66.0504 224.065 -63.9308 239.41 -58.5361C254.754 -53.1414 268.832 -44.5878 280.757 -33.4124C292.682 -22.2369 302.196 -8.68194 308.702 6.39988C315.134 21.3138 318.483 37.4019 318.551 53.6733L298.696 51.919C298.455 38.7552 295.611 25.7724 290.326 13.7409L290.103 13.2307L289.625 13.5089L272.26 23.6428L271.882 23.8634L272.048 24.2711C278.514 40.1402 279.638 57.7262 275.245 74.3061C270.901 90.7012 261.401 105.207 248.194 115.632L238.415 98.4753C246.945 91.0702 253.16 81.3016 256.287 70.3626C259.453 59.2887 259.315 47.5128 255.892 36.5169C252.469 25.5209 245.912 15.7962 237.048 8.56694C228.292 1.42595 217.67 -2.9636 206.491 -4.06957L204.803 -23.8035C215.835 -23.2382 226.622 -20.2771 236.429 -15.1154L236.43 -15.1156C246.405 -9.8657 255.126 -2.46736 261.982 6.5593L262.247 6.9086L262.624 6.68831L280.046 -3.49542L280.522 -3.7744L280.2 -4.2262C267.329 -22.247 248.885 -35.3926 227.798 -41.5743C206.712 -47.7558 184.193 -46.6194 163.82 -38.3444C143.447 -30.0694 126.388 -15.1307 115.354 4.09694C104.394 23.1968 100.004 45.441 102.865 67.338L84.5078 72.3214C82.0618 56.2426 82.8841 39.8252 86.9313 24.0789C91.0248 8.15219 98.3266 -6.74338 108.377 -19.6706C118.427 -32.5979 131.01 -43.2767 145.331 -51.0364Z"
							stroke="#1BD96A"
							stroke-opacity="0.12"
						/>
						<path
							d="M260.003 157.491C244.923 166.318 228.106 171.665 210.752 173.15C193.397 174.636 175.935 172.223 159.61 166.084C143.286 159.945 128.503 150.231 116.318 137.637C104.132 125.042 94.8439 109.878 89.1171 93.226L108.448 87.9782C110.395 93.3018 112.784 98.4486 115.59 103.363C118.525 108.52 121.913 113.398 125.713 117.939L140.152 102.814C131.996 92.3742 126.591 80.0086 124.444 66.8751C122.296 53.7416 123.476 40.2699 127.873 27.7219C132.27 15.1739 139.74 3.96017 149.584 -4.86882C159.427 -13.6978 171.322 -19.8532 184.154 -22.7584L185.891 -2.02536C177.437 0.311457 169.624 4.57902 163.052 10.4495C156.48 16.3201 151.323 23.6373 147.979 31.8393C144.634 40.0412 143.191 48.9096 143.759 57.7633C144.327 66.6169 146.892 75.2202 151.257 82.9123C152.243 84.6198 153.258 86.3022 154.382 87.7452L172.854 68.4135L161.573 52.4568L176.638 25.2047L201.398 12.3472L211.468 19.805L202.636 35.5055L192.974 41.7298L187.498 51.6422L193.955 61.0422C193.955 61.0422 203.56 67.0702 203.576 67.0659L213.41 61.3547L218.72 50.9454L233.753 41.2004L241.537 51.0445L230.214 76.6512L204.201 93.4501L187.642 82.5445L169.003 102.096C176.464 107.133 184.988 110.331 193.89 111.432C202.792 112.534 211.826 111.509 220.268 108.44L230.553 126.503C218.179 131.679 204.694 133.531 191.407 131.879C178.121 130.227 165.481 125.128 154.715 117.075L140.327 132.134C153.557 142.488 169.184 149.244 185.722 151.759C202.26 154.274 219.16 152.465 234.815 146.503C250.471 140.542 264.362 130.626 275.169 117.699C285.976 104.771 293.339 89.2611 296.56 72.6427L317.419 74.4794C314.438 91.7283 307.75 108.104 297.828 122.449C287.906 136.794 274.993 148.756 260.003 157.491Z"
							fill="#1BD96A"
							fill-opacity="0.16"
						/>
						<path
							d="M149.913 -4.49238C159.551 -13.1371 171.169 -19.2006 183.706 -22.1377L185.36 -2.39778C176.987 -0.0177107 169.248 4.24324 162.723 10.0719C156.094 15.9933 150.893 23.3739 147.519 31.6468C144.146 39.9199 142.69 48.8658 143.263 57.7963C143.837 66.7266 146.424 75.4045 150.826 83.1632L150.828 83.1668C151.816 84.8756 152.845 86.5845 153.993 88.0571L154.344 88.5081L154.739 88.0956L173.211 68.7634L173.5 68.4621L173.258 68.1208L162.16 52.4243L176.998 25.5829L201.351 12.9363L210.816 19.9464L202.265 35.1485L192.707 41.3047L192.602 41.373L192.541 41.4842L187.064 51.3967L186.912 51.6708L187.09 51.9298L193.547 61.3299L193.606 61.4157L193.693 61.4702L193.7 61.4744C193.705 61.4773 193.712 61.4815 193.721 61.4871C193.739 61.4985 193.766 61.5158 193.801 61.5377C193.871 61.5819 193.975 61.6461 194.106 61.7285C194.369 61.8933 194.744 62.1297 195.194 62.4122C196.095 62.9772 197.296 63.7303 198.498 64.4836C199.7 65.2368 200.903 65.9902 201.806 66.5549C202.257 66.8371 202.634 67.0726 202.898 67.2372C203.03 67.3195 203.136 67.3844 203.208 67.4289C203.244 67.4509 203.273 67.4687 203.293 67.4811C203.303 67.487 203.312 67.4932 203.321 67.498C203.324 67.5001 203.332 67.5044 203.341 67.5089C203.344 67.5108 203.354 67.516 203.367 67.522C203.375 67.5257 203.397 67.5353 203.411 67.5406C203.444 67.5507 203.59 67.5693 203.705 67.5525L203.767 67.5355L203.823 67.5021L213.656 61.7911L213.783 61.7179L213.851 61.5856L219.099 51.2968L233.644 41.8683L240.959 51.1188L229.821 76.3076L204.202 92.8511L187.912 82.1225L187.569 81.8961L187.285 82.1952L168.646 101.746L168.233 102.18L168.728 102.515C176.254 107.596 184.85 110.821 193.829 111.932C202.671 113.026 211.64 112.038 220.043 109.052L229.836 126.252C217.685 131.229 204.482 132.997 191.468 131.379C178.266 129.738 165.708 124.671 155.01 116.67L154.66 116.409L154.358 116.725L139.97 131.784L139.584 132.189L140.024 132.532C153.321 142.939 169.026 149.729 185.648 152.257C202.269 154.785 219.255 152.966 234.99 146.974C250.724 140.983 264.686 131.017 275.548 118.024C286.313 105.146 293.676 89.7174 296.957 73.1823L316.833 74.9331C313.819 91.9106 307.198 108.026 297.421 122.16C287.541 136.444 274.683 148.357 259.755 157.055L259.754 157.055C244.738 165.845 227.991 171.169 210.71 172.648C193.429 174.128 176.039 171.725 159.783 165.612C143.528 159.499 128.806 149.826 116.672 137.284C104.662 124.872 95.4819 109.951 89.7657 93.5707L108.141 88.5822C109.945 93.4461 112.116 98.1617 114.637 102.686L115.16 103.615C118.11 108.797 121.515 113.7 125.333 118.264L125.689 118.688L126.07 118.289L140.509 103.163L140.811 102.847L140.541 102.501C132.438 92.1288 127.067 79.8422 124.933 66.7929C122.799 53.7434 123.973 40.3579 128.341 27.8902C132.71 15.4225 140.132 4.28013 149.913 -4.49238Z"
							stroke="#1BD96A"
							stroke-opacity="0.12"
						/>
					</svg>
				</button>
			</div>
			<span v-if="hasTinMismatch" class="text-sm font-semibold text-red">
				{{ formatMessage(messages.withdrawBlockedTinMismatch) }}
			</span>
		</div>

		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-2 sm:flex-row sm:justify-between">
				<span class="text-xl font-semibold text-contrast md:text-2xl">{{
					formatMessage(messages.transactionsHeader)
				}}</span>
				<nuxt-link
					v-if="sortedPayouts.length > 0"
					class="mt-0 font-semibold text-contrast underline underline-offset-2 sm:my-auto"
					to="/dashboard/revenue/transfers"
					>{{ formatMessage(messages.seeAll) }}</nuxt-link
				>
			</div>
			<div v-if="sortedPayouts.length > 0" class="flex flex-col gap-3 md:gap-4">
				<RevenueTransaction
					v-for="transaction in sortedPayouts.slice(0, 3)"
					:key="transaction.id || transaction.created"
					:transaction="transaction"
					@cancelled="refreshPayouts"
				/>
			</div>
			<div v-else class="mx-auto flex flex-col justify-center gap-8 p-6 text-center">
				<div data-svg-wrapper>
					<svg
						viewBox="0 0 154 94"
						fill="none"
						class="h-[94px] w-[154px]"
						xmlns="http://www.w3.org/2000/svg"
					>
						<path
							fill-rule="evenodd"
							clip-rule="evenodd"
							d="M147.545 13.8413C151.11 13.8413 154 16.7333 154 20.3006C154 23.868 151.11 26.76 147.545 26.76H110.659C114.224 26.76 117.114 29.6519 117.114 33.2193C117.114 36.7867 114.224 39.6786 110.659 39.6786H130.946C134.511 39.6786 137.401 42.5706 137.401 46.138C137.401 49.7054 134.511 52.5973 130.946 52.5973H121.564C117.069 52.5973 113.425 55.4892 113.425 59.0566C113.425 61.4349 115.269 63.588 118.958 65.516C122.523 65.516 125.413 68.4079 125.413 71.9753C125.413 75.5427 122.523 78.4346 118.958 78.4346H42.4192C38.8541 78.4346 35.9641 75.5427 35.9641 71.9753C35.9641 68.4079 38.8541 65.516 42.4192 65.516H6.45509C2.89004 65.516 0 62.624 0 59.0566C0 55.4892 2.89004 52.5973 6.45509 52.5973H43.3413C46.9064 52.5973 49.7964 49.7054 49.7964 46.138C49.7964 42.5706 46.9064 39.6786 43.3413 39.6786H20.2874C16.7224 39.6786 13.8323 36.7867 13.8323 33.2193C13.8323 29.6519 16.7224 26.76 20.2874 26.76H57.1737C53.6086 26.76 50.7186 23.868 50.7186 20.3006C50.7186 16.7333 53.6086 13.8413 57.1737 13.8413H147.545ZM147.545 39.6786C151.11 39.6786 154 42.5706 154 46.138C154 49.7054 151.11 52.5973 147.545 52.5973C143.98 52.5973 141.09 49.7054 141.09 46.138C141.09 42.5706 143.98 39.6786 147.545 39.6786Z"
							fill="var(--surface-2, #1D1F23)"
						/>
						<path
							fill-rule="evenodd"
							clip-rule="evenodd"
							d="M98.3725 12.9186L106.946 75.5171L107.717 81.7973C107.965 83.8205 106.527 85.6621 104.505 85.9107L50.499 92.5499C48.4769 92.7984 46.6365 91.3599 46.3883 89.3367L38.0786 21.585C37.9545 20.5734 38.6735 19.6526 39.6846 19.5283C39.691 19.5275 39.6974 19.5267 39.7038 19.526L44.1843 19.0228M47.8065 18.612L52.0365 18.1369Z"
							fill="var(--surface-1, #16181C)"
						/>
						<path
							d="M99.6109 12.7488C99.5172 12.0648 98.8868 11.5864 98.2028 11.6802C97.5189 11.774 97.0404 12.4045 97.1341 13.0884L98.3725 12.9186L99.6109 12.7488ZM106.946 75.5171L108.187 75.3647C108.186 75.3589 108.186 75.3531 108.185 75.3473L106.946 75.5171ZM107.717 81.7973L108.957 81.6449L107.717 81.7973ZM104.505 85.9107L104.657 87.1514L104.505 85.9107ZM50.499 92.5499L50.6514 93.7905L50.499 92.5499ZM46.3883 89.3367L47.629 89.1843L46.3883 89.3367ZM38.0786 21.585L36.8379 21.7374L36.8379 21.7374L38.0786 21.585ZM39.6846 19.5283L39.8369 20.769L39.6846 19.5283ZM39.7038 19.526L39.8431 20.7682L39.7038 19.526ZM44.3236 20.265C45.0096 20.1879 45.5034 19.5693 45.4265 18.8833C45.3495 18.1972 44.731 17.7035 44.0449 17.7806L44.1843 19.0228L44.3236 20.265ZM47.6672 17.3698C46.9811 17.4468 46.4874 18.0654 46.5643 18.7515C46.6412 19.4375 47.2598 19.9312 47.9458 19.8542L47.8065 18.612L47.6672 17.3698ZM52.1758 19.3791C52.8619 19.3021 53.3557 18.6835 53.2787 17.9974C53.2018 17.3114 52.5833 16.8177 51.8972 16.8947L52.0365 18.1369L52.1758 19.3791ZM98.3725 12.9186L97.1341 13.0884L105.708 75.6869L106.946 75.5171L108.185 75.3473L99.6109 12.7488L98.3725 12.9186ZM106.946 75.5171L105.706 75.6695L106.476 81.9497L107.717 81.7973L108.957 81.6449L108.187 75.3647L106.946 75.5171ZM107.717 81.7973L106.476 81.9497C106.64 83.2883 105.689 84.5057 104.352 84.67L104.505 85.9107L104.657 87.1514C107.365 86.8185 109.29 84.3527 108.957 81.6449L107.717 81.7973ZM104.505 85.9107L104.352 84.67L50.3467 91.3092L50.499 92.5499L50.6514 93.7905L104.657 87.1514L104.505 85.9107ZM50.499 92.5499L50.3467 91.3092C49.0104 91.4734 47.7932 90.5228 47.629 89.1843L46.3883 89.3367L45.1476 89.489C45.4798 92.1969 47.9434 94.1234 50.6514 93.7905L50.499 92.5499ZM46.3883 89.3367L47.629 89.1843L39.3192 21.4326L38.0786 21.585L36.8379 21.7374L45.1476 89.489L46.3883 89.3367ZM38.0786 21.585L39.3192 21.4326C39.2791 21.1056 39.5116 20.809 39.8369 20.769L39.6846 19.5283L39.5323 18.2876C37.8355 18.4962 36.6298 20.0412 36.8379 21.7374L38.0786 21.585ZM39.6846 19.5283L39.8369 20.769C39.839 20.7687 39.841 20.7685 39.8431 20.7682L39.7038 19.526L39.5645 18.2838C39.5537 18.285 39.543 18.2863 39.5323 18.2876L39.6846 19.5283ZM39.7038 19.526L39.8431 20.7682L44.3236 20.265L44.1843 19.0228L44.0449 17.7806L39.5645 18.2838L39.7038 19.526ZM47.8065 18.612L47.9458 19.8542L52.1758 19.3791L52.0365 18.1369L51.8972 16.8947L47.6672 17.3698L47.8065 18.612Z"
							fill="var(--surface-4, #34363C)"
						/>
						<path
							fill-rule="evenodd"
							clip-rule="evenodd"
							d="M96.0333 16.8579L103.797 73.5927L104.496 79.2845C104.721 81.1182 103.435 82.785 101.625 83.0074L53.2649 88.9492C51.4542 89.1716 49.8039 87.8655 49.5789 86.0319L42.0821 24.9356C41.9476 23.8392 42.7273 22.8412 43.8236 22.7065L49.6517 21.9905"
							fill="var(--surface-2, #1D1F23)"
						/>
						<path
							d="M59.0267 1.25H100.596C101.325 1.25 102.025 1.53995 102.54 2.05566L114.756 14.2715C115.272 14.7872 115.561 15.4867 115.561 16.2158V73.5117C115.561 75.0305 114.33 76.2617 112.811 76.2617H59.0267C57.508 76.2617 56.2767 75.0305 56.2767 73.5117V4C56.2767 2.48122 57.508 1.25 59.0267 1.25Z"
							fill="var(--surface-1, #16181C)"
							stroke="var(--surface-4, #34363C)"
							stroke-width="2.5"
						/>
						<path
							d="M101.135 2.21729V12.9187C101.135 14.4476 102.373 15.687 103.901 15.687H111.217"
							stroke="var(--surface-4, #34363C)"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
						<path
							d="M65.473 62.7479H89.4491M65.473 15.687H89.4491H65.473ZM65.473 26.7602H105.126H65.473ZM65.473 38.7561H105.126H65.473ZM65.473 50.752H105.126H65.473Z"
							stroke="var(--surface-3, #27292E)"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
					</svg>
				</div>
				<div class="flex flex-col gap-1">
					<span class="text-lg text-contrast md:text-xl">{{
						formatMessage(messages.noTransactions)
					}}</span>
					<span class="max-w-[256px] text-lg leading-6 text-secondary">{{
						formatMessage(messages.noTransactionsDesc)
					}}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ArrowUpRightIcon, InProgressIcon, UnknownIcon } from '@modrinth/assets'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'
import { Tooltip } from 'floating-vue'

import { useUserCountry } from '@/composables/country.ts'
import type { PayoutMethod } from '@/providers/creator-withdraw.ts'
import CreatorWithdrawModal from '~/components/ui/dashboard/CreatorWithdrawModal.vue'
import RevenueTransaction from '~/components/ui/dashboard/RevenueTransaction.vue'

const { formatMessage } = useVIntl()

await useAuth()

// TODO: Deduplicate these types & interfaces in @modrinth/api-client PR.
type FormCompletionStatus = 'unknown' | 'unrequested' | 'unsigned' | 'tin-mismatch' | 'complete'

type UserBalanceResponse = {
	available: number
	withdrawn_lifetime: number
	withdrawn_ytd: number
	pending: number
	// ISO 8601 date string -> amount
	dates: Record<string, number>
	// backend returns null when not applicable
	requested_form_type: string | null
	form_completion_status: FormCompletionStatus | null
}

type RevenueBarSegment = {
	key: string
	class: string
	widthPct: string
	amount: number
}

const hoveredSeg = ref<string | null>(null)

const withdrawModal = ref<InstanceType<typeof CreatorWithdrawModal>>()
async function openWithdrawModal() {
	withdrawModal.value?.show?.()
}

const messages = defineMessages({
	balanceLabel: { id: 'dashboard.revenue.balance', defaultMessage: 'Balance' },
	availableNow: { id: 'dashboard.revenue.available-now', defaultMessage: 'Available now' },
	estimatedWithDate: {
		id: 'dashboard.revenue.estimated-with-date',
		defaultMessage: 'Estimated {date}',
	},
	estimatedTooltip1: {
		id: 'dashboard.revenue.estimated-tooltip.msg1',
		defaultMessage: 'Estimated revenue may be subject to change until it is made available.',
	},
	estimatedTooltip2: {
		id: 'dashboard.revenue.estimated-tooltip.msg2',
		defaultMessage: 'Click to read about how Modrinth handles your revenue.',
	},
	processing: { id: 'dashboard.revenue.processing', defaultMessage: 'Processing' },
	processingTooltip: {
		id: 'dashboard.revenue.processing.tooltip',
		defaultMessage:
			'Revenue stays in processing until the end of the month, then becomes available 60 days later.',
	},
	withdrawHeader: { id: 'dashboard.revenue.withdraw.header', defaultMessage: 'Withdraw' },
	withdrawCardTitle: { id: 'dashboard.revenue.withdraw.card.title', defaultMessage: 'Withdraw' },
	withdrawCardDescription: {
		id: 'dashboard.revenue.withdraw.card.description',
		defaultMessage: 'Withdraw from your available balance to any payout method.',
	},
	withdrawBlockedTinMismatch: {
		id: 'dashboard.revenue.withdraw.blocked-tin-mismatch',
		defaultMessage:
			"Your withdrawals are temporarily locked because your TIN or SSN didn't match IRS records. Please contact support to reset and resubmit your tax form.",
	},
	tosLabel: {
		id: 'dashboard.revenue.tos',
		defaultMessage:
			'By uploading projects to Modrinth and withdrawing money from your account, you agree to our <terms-link>Rewards Program Terms</terms-link>. Learn more about the <info-link>Reward Program</info-link>.',
	},
	transactionsHeader: {
		id: 'dashboard.revenue.transactions.header',
		defaultMessage: 'Transactions',
	},
	seeAll: { id: 'dashboard.revenue.transactions.see-all', defaultMessage: 'See all' },
	noTransactions: {
		id: 'dashboard.revenue.transactions.none',
		defaultMessage: 'No transactions',
	},
	noTransactionsDesc: {
		id: 'dashboard.revenue.transactions.none.desc',
		defaultMessage: 'Your payouts and withdrawals will appear here.',
	},
})

const { data: userBalance, refresh: refreshUserBalance } = await useAsyncData(
	`payout/balance`,
	async () => {
		const response = (await useBaseFetch(`payout/balance`, {
			apiVersion: 3,
		})) as UserBalanceResponse
		return {
			...response,
			available: Number(response.available),
			withdrawn_lifetime: Number(response.withdrawn_lifetime),
			withdrawn_ytd: Number(response.withdrawn_ytd),
			pending: Number(response.pending),
		}
	},
)

const { data: payouts, refresh: refreshPayouts } = await useAsyncData(`payout/history`, () =>
	useBaseFetch(`payout/history`, {
		apiVersion: 3,
	}),
)

const userCountry = useUserCountry()
const { data: preloadedPaymentMethods } = await useAsyncData(`payout/methods-preload`, async () => {
	const defaultCountry = userCountry.value || 'US'
	try {
		return {
			country: defaultCountry,
			methods: (await useBaseFetch('payout/methods', {
				apiVersion: 3,
				query: { country: defaultCountry },
			})) as PayoutMethod[],
		}
	} catch {
		return null
	}
})

const sortedPayouts = computed(() => {
	if (!payouts.value) return []

	return [...payouts.value].sort((a, b) => {
		return new Date(b.created).getTime() - new Date(a.created).getTime()
	})
})

const totalAvailable = computed(() => (userBalance.value ? userBalance.value.available : 0))
const nextDate = computed<{ date: string; amount: number }[]>(() => {
	const dates = userBalance.value?.dates
	if (!dates) return []

	const now = Date.now()

	return Object.entries(dates)
		.map(([date, amount]) => ({ date, amount: Number(amount) }))
		.filter(({ date }) => new Date(date).getTime() > now)
		.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime())
})

const processingDate = computed<{ date: string; amount: number }>(() => {
	const nextDates = nextDate.value
	if (!nextDates.length) return { date: '', amount: 0 }

	const now = dayjs.utc()
	const currentMonth = now.format('YYYY-MM')

	// Find revenue from the current month (still "processing")
	// Revenue earned in month X becomes available at end_of_month(X) + 60 days
	// So we calculate: source_month = (date_available - 60 days).startOf('month')
	for (const { date, amount } of nextDates) {
		const availableDate = dayjs.utc(date)
		const sourceMonthEnd = availableDate.subtract(60, 'days')
		const sourceMonth = sourceMonthEnd.startOf('month').format('YYYY-MM')

		// If this revenue is from the current month, it's still "processing"
		if (sourceMonth === currentMonth) {
			return { date, amount: Number(amount) }
		}
	}

	// No revenue from current month found
	return { date: '', amount: 0 }
})

const grandTotal = computed(() =>
	userBalance.value ? userBalance.value.available + userBalance.value.pending : 0,
)

const hasTinMismatch = computed(() => {
	const bal = userBalance.value
	if (!bal) return false
	const status = bal.form_completion_status ?? 'unknown'
	return status === 'tin-mismatch'
})

async function refreshData() {
	try {
		await Promise.all([refreshUserBalance(), refreshPayouts()])
	} catch (error) {
		console.error('Failed to refresh data:', error)
	}
}

const dateStripeClasses = [
	'zone--striped--blue bg-gradient-to-b from-[#4E9CFF] to-[#4181D3]',
	'zone--striped--purple bg-gradient-to-b from-[#c084fc] to-[#a855f7]',
	'zone--striped--orange bg-gradient-to-b from-[#fb923c] to-[#f97316]',
	'zone--striped--red bg-gradient-to-b from-[#f87171] to-[#ef4444]',
] as const

const dateHighlightClasses = [
	'bg-highlight-blue',
	'bg-highlight-purple',
	'bg-highlight-orange',
	'bg-highlight-red',
] as const

const dateTextClasses = ['!text-brand-blue', '!text-purple', '!text-orange', '!text-red'] as const

const dateSegments = computed(() => {
	const dates = nextDate.value
	if (!dates?.length)
		return [] as Array<{
			date: string
			amount: number
			stripeClass: string
			highlightClass: string
			textClass: string
		}>

	const processing = processingDate.value

	// Filter out the processing date (current month's revenue)
	// Show only finalized pending dates as "Estimated"
	const estimatedDates = processing.date ? dates.filter((d) => d.date !== processing.date) : dates

	return estimatedDates.map((d, i) => ({
		...d,
		stripeClass: dateStripeClasses[i % dateStripeClasses.length],
		highlightClass: dateHighlightClasses[i % dateHighlightClasses.length],
		textClass: dateTextClasses[i % dateTextClasses.length],
	}))
})

const segments = computed<RevenueBarSegment[]>(() => {
	const available = totalAvailable.value || 0
	const dates = nextDate.value || []
	const processing = processingDate.value

	// Filter out processing date from upcoming dates (same logic as dateSegments)
	const upcoming = processing.date ? dates.filter((d) => d.date !== processing.date) : dates

	const totalPending = userBalance.value?.pending ?? 0
	const total = available + totalPending

	if (total <= 0) return [] as RevenueBarSegment[]

	const segs: Array<{ key: string; class: string; width: number; amount: number }> = []

	if (available > 0) {
		segs.push({
			key: 'available',
			class: 'bg-gradient-to-b from-[#1CD96A] to-[#17B257]',
			width: available / total,
			amount: available,
		})
	}

	upcoming.forEach((d, i) => {
		const amt = Number(d.amount) || 0
		if (amt <= 0) return
		const stripe = dateStripeClasses[i % dateStripeClasses.length]
		const hi = dateHighlightClasses[i % dateHighlightClasses.length]
		segs.push({
			key: `upcoming-${d.date}-${i}`,
			class: `${stripe} ${hi}`,
			width: amt / total,
			amount: amt,
		})
	})

	// Always show processing section (even if $0)
	const processingAmt = Number(processing.amount) || 0
	segs.push({
		key: 'processing',
		class: 'zone--striped--gray bg-button-bg',
		width: processingAmt / total,
		amount: processingAmt,
	})

	let acc = 0
	const normalized = segs.map((s, idx) => {
		let pct = Math.round(s.width * 10000) / 100
		if (idx === segs.length - 1) {
			pct = Math.max(0, 100 - acc)
		}
		acc += pct
		return { key: s.key, class: s.class, pct, amount: s.amount }
	})

	const filtered = normalized.filter((s) => s.pct >= 1)
	if (!filtered.length) return [] as RevenueBarSegment[]

	const sumExceptLast = filtered.slice(0, -1).reduce((sum, s) => sum + s.pct, 0)
	filtered[filtered.length - 1].pct = Math.max(0, 100 - sumExceptLast)

	return filtered.map((s) => ({
		key: s.key,
		class: s.class,
		widthPct: `${s.pct}%`,
		amount: s.amount,
	})) as RevenueBarSegment[]
})
</script>

<style scoped lang="scss">
%zone--striped-common {
	/* Use scroll so stripes remain static relative to element when page scrolls */
	background-attachment: scroll;
	background-position: 0 0;
	background-size: 9.38px 9.38px;
}

@mixin striped-background($color-variable) {
	background-image: linear-gradient(
		135deg,
		$color-variable 11.54%,
		transparent 11.54%,
		transparent 50%,
		$color-variable 50%,
		$color-variable 61.54%,
		transparent 61.54%,
		transparent 100%
	);
}

$striped-colors: 'green', 'blue', 'purple', 'orange', 'red';

@each $color in $striped-colors {
	.zone--striped--#{$color} {
		@include striped-background(var(--color-#{$color}));
		@extend %zone--striped-common;
	}
}

.zone--striped--gray {
	@extend %zone--striped-common;
	background-image: linear-gradient(
		135deg,
		var(--color-divider-dark) 11.54%,
		transparent 11.54%,
		transparent 50%,
		var(--color-divider-dark) 50%,
		var(--color-divider-dark) 61.54%,
		transparent 61.54%,
		transparent 100%
	);
}

.zone--striped-small {
	background-size: 6.19px 6.19px !important;
	background-position: unset !important;
	background-attachment: unset !important;
}

$flash-colors: 'green', 'blue', 'purple', 'orange', 'red', 'gray';

@each $color in $flash-colors {
	@keyframes flash-#{$color} {
		0%,
		100% {
			color: var(--color-contrast);
		}

		50% {
			color: var(--color-#{$color});
		}
	}

	.animate-flash-#{$color} {
		animation: flash-#{$color} 1.5s ease-in-out infinite;
	}

	.text-#{$color}.animate-flash-color {
		animation: flash-#{$color} 1.5s ease-in-out infinite;
	}
}

.gradient-border {
	position: relative;

	&::after {
		content: '';
		position: absolute;
		inset: 0;
		background: linear-gradient(to bottom, rgba(255, 255, 255, 0.3), transparent);
		border-radius: inherit;
		mask:
			linear-gradient(#fff 0 0) content-box,
			linear-gradient(#fff 0 0);
		mask-composite: xor;
		padding: 2px;
		pointer-events: none;
	}
}
</style>

<style>
.v-popper--theme-dismissable-prompt .v-popper__inner {
	border-color: transparent !important;
}

.v-popper--theme-dismissable-prompt .v-popper__arrow-outer,
.v-popper--theme-dismissable-prompt .v-popper__arrow-inner {
	border-color: transparent !important;
}
</style>
