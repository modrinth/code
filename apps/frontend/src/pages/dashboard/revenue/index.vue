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
					class="flex flex-row justify-between border-0 !border-b-[2px] border-solid border-button-bg p-1.5 md:p-2"
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
					class="flex flex-row justify-between border-0 !border-b-[2px] border-solid border-button-bg p-1.5 md:p-2"
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
					class="relative flex flex-col overflow-hidden rounded-2xl bg-gradient-to-r from-green to-green-700 p-4 text-inverted shadow-md transition-all duration-200 hover:brightness-105 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:brightness-100 md:p-5"
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
			<div v-else class="mx-auto flex flex-col justify-center p-6 text-center">
				<div data-svg-wrapper className="relative w-full">
					<svg
						width="250"
						height="200"
						viewBox="0 0 250 200"
						fill="none"
						xmlns="http://www.w3.org/2000/svg"
						class="h-auto w-full"
					>
						<rect width="250" height="200" fill="var(--surface-1)" />
						<path
							fill-rule="evenodd"
							clip-rule="evenodd"
							d="M207 65C210.866 65 214 68.134 214 72C214 75.866 210.866 79 207 79H167C170.866 79 174 82.134 174 86C174 89.866 170.866 93 167 93H189C192.866 93 196 96.134 196 100C196 103.866 192.866 107 189 107H178.826C173.952 107 170 110.134 170 114C170 116.577 172 118.911 176 121C179.866 121 183 124.134 183 128C183 131.866 179.866 135 176 135H93C89.134 135 86 131.866 86 128C86 124.134 89.134 121 93 121H54C50.134 121 47 117.866 47 114C47 110.134 50.134 107 54 107H94C97.866 107 101 103.866 101 100C101 96.134 97.866 93 94 93H69C65.134 93 62 89.866 62 86C62 82.134 65.134 79 69 79H109C105.134 79 102 75.866 102 72C102 68.134 105.134 65 109 65H207ZM207 93C210.866 93 214 96.134 214 100C214 103.866 210.866 107 207 107C203.134 107 200 103.866 200 100C200 96.134 203.134 93 207 93Z"
							fill="var(--surface-2)"
						/>
						<path
							fill-rule="evenodd"
							clip-rule="evenodd"
							d="M153.672 63.9999L162.974 131.843L163.809 138.649C164.079 140.842 162.519 142.837 160.327 143.107L101.767 150.297C99.5739 150.566 97.5781 149.007 97.3089 146.814L88.2931 73.3867C88.1585 72.2904 88.9381 71.2925 90.0345 71.1579C90.0414 71.157 90.0483 71.1562 90.0553 71.1554L94.9136 70.6104M98.8422 70.1698L103.429 69.6552L98.8422 70.1698Z"
							fill="var(--surface-1)"
						/>
						<path
							d="M154.91 63.8302C154.817 63.1462 154.186 62.6678 153.502 62.7615C152.818 62.8553 152.34 63.4858 152.433 64.1697L153.672 63.9999L154.91 63.8302ZM162.974 131.843L164.214 131.69C164.214 131.685 164.213 131.679 164.212 131.673L162.974 131.843ZM163.809 138.649L165.05 138.497L163.809 138.649ZM160.327 143.107L160.479 144.347L160.327 143.107ZM101.767 150.297L101.919 151.538L101.767 150.297ZM97.3089 146.814L98.5496 146.662L97.3089 146.814ZM90.0553 71.1554L90.1946 72.3976L90.0553 71.1554ZM95.053 71.8527C95.739 71.7757 96.2328 71.1572 96.1558 70.4711C96.0789 69.7851 95.4603 69.2913 94.7743 69.3682L94.9136 70.6104L95.053 71.8527ZM98.7028 68.9276C98.0168 69.0045 97.523 69.6231 97.5999 70.3091C97.6769 70.9952 98.2954 71.4889 98.9815 71.412L98.8422 70.1698L98.7028 68.9276ZM103.569 70.8974C104.255 70.8205 104.748 70.2019 104.671 69.5159C104.594 68.8298 103.976 68.3361 103.29 68.413L103.429 69.6552L103.569 70.8974ZM153.672 63.9999L152.433 64.1697L161.735 132.012L162.974 131.843L164.212 131.673L154.91 63.8302L153.672 63.9999ZM162.974 131.843L161.733 131.995L162.569 138.801L163.809 138.649L165.05 138.497L164.214 131.69L162.974 131.843ZM163.809 138.649L162.569 138.801C162.754 140.309 161.682 141.681 160.174 141.866L160.327 143.107L160.479 144.347C163.357 143.994 165.404 141.375 165.05 138.497L163.809 138.649ZM160.327 143.107L160.174 141.866L101.614 149.056L101.767 150.297L101.919 151.538L160.479 144.347L160.327 143.107ZM101.767 150.297L101.614 149.056C100.107 149.241 98.7347 148.169 98.5496 146.662L97.3089 146.814L96.0682 146.967C96.4216 149.844 99.041 151.891 101.919 151.538L101.767 150.297ZM97.3089 146.814L98.5496 146.662L89.5338 73.2344L88.2931 73.3867L87.0524 73.539L96.0682 146.967L97.3089 146.814ZM88.2931 73.3867L89.5338 73.2344C89.4833 72.8232 89.7757 72.449 90.1868 72.3986L90.0345 71.1579L89.8821 69.9172C88.1006 70.1359 86.8337 71.7575 87.0524 73.539L88.2931 73.3867ZM90.0345 71.1579L90.1868 72.3986C90.1894 72.3982 90.192 72.3979 90.1946 72.3976L90.0553 71.1554L89.9159 69.9132C89.9046 69.9145 89.8934 69.9158 89.8821 69.9172L90.0345 71.1579ZM90.0553 71.1554L90.1946 72.3976L95.053 71.8527L94.9136 70.6104L94.7743 69.3682L89.9159 69.9132L90.0553 71.1554ZM98.8422 70.1698L98.9815 71.412L103.569 70.8974L103.429 69.6552L103.29 68.413L98.7028 68.9276L98.8422 70.1698Z"
							fill="var(--surface-4)"
						/>
						<path
							fill-rule="evenodd"
							clip-rule="evenodd"
							d="M151.14 68.2691L159.56 129.753L160.317 135.921C160.561 137.908 159.167 139.714 157.203 139.956L104.761 146.395C102.798 146.636 101.008 145.22 100.764 143.233L92.6142 76.8567C92.4795 75.7603 93.2592 74.7625 94.3555 74.6278L100.843 73.8313"
							fill="var(--surface-2)"
						/>
						<path
							d="M110.672 51.25H156.229C156.958 51.25 157.657 51.5393 158.173 52.0547L171.616 65.4902C172.132 66.0059 172.422 66.7053 172.422 67.4346V130C172.422 131.519 171.191 132.75 169.672 132.75H110.672C109.153 132.75 107.922 131.519 107.922 130V54C107.922 52.4812 109.153 51.25 110.672 51.25Z"
							fill="var(--surface-1)"
							stroke="var(--surface-4)"
							stroke-width="2.5"
						/>
						<path
							d="M156.672 52.4028V64C156.672 65.6569 158.015 67 159.672 67H167.605"
							stroke="var(--surface-4)"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
						<path
							d="M118 118H144M118 67H144H118ZM118 79H161H118ZM118 92H161H118ZM118 105H161H118Z"
							stroke="var(--surface-3)"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
					</svg>
				</div>
				<span class="text-lg text-contrast md:text-xl">{{
					formatMessage(messages.noTransactions)
				}}</span>
				<span class="max-w-[256px] text-lg leading-none text-secondary">{{
					formatMessage(messages.noTransactionsDesc)
				}}</span>
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

	const now = dayjs()
	const currentMonth = now.format('YYYY-MM')

	// Find revenue from the current month (still "processing")
	// Revenue earned in month X becomes available at end_of_month(X) + 60 days
	// So we calculate: source_month = (date_available - 60 days).startOf('month')
	for (const { date, amount } of nextDates) {
		const availableDate = dayjs(date)
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

	const filtered = normalized.filter((s) => s.pct > 0)
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
