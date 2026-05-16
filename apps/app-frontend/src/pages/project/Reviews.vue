<template>
	<div class="flex flex-col gap-4">
		<section class="universal-card flex flex-col gap-4">
			<div class="flex items-center justify-between">
				<div>
					<h2 class="m-0 text-lg font-bold text-contrast">Reviews</h2>
					<p v-if="reviewsData" class="m-0 text-sm text-secondary">
						{{ reviewsData.total }} review{{ reviewsData.total === 1 ? '' : 's' }}
						<template v-if="averageRating !== null">
							&middot; {{ averageRating.toFixed(1) }} / 5 average
						</template>
					</p>
				</div>
				<ButtonStyled
					v-if="auth?.user && !ownReview && !showForm"
					color="brand"
					@click="showForm = true"
				>
					<button type="button">
						<StarIcon aria-hidden="true" />
						Write a Review
					</button>
				</ButtonStyled>
			</div>

			<div v-if="averageRating !== null && reviewsData && reviewsData.total > 0" class="flex items-center gap-3">
				<div class="flex gap-0.5">
					<StarIcon
						v-for="i in 5"
						:key="i"
						class="h-5 w-5"
						:class="i <= Math.round(averageRating) ? 'text-yellow-400' : 'text-secondary'"
						aria-hidden="true"
					/>
				</div>
				<span class="text-xl font-bold text-contrast">{{ averageRating.toFixed(1) }}</span>
				<span class="text-secondary">out of 5</span>
			</div>

			<template v-if="showForm || ownReview">
				<hr class="m-0 border-button-border" />
				<div class="flex flex-col gap-3">
					<h3 class="m-0 text-base font-semibold text-contrast">
						{{ ownReview ? 'Your review' : 'Write a review' }}
					</h3>

					<div class="flex flex-col gap-2">
						<label class="text-sm font-medium text-secondary">Rating</label>
						<div class="flex gap-1">
							<button
								v-for="i in 5"
								:key="i"
								type="button"
								class="rounded p-0.5 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand"
								:aria-label="`Rate ${i} out of 5`"
								@click="formRating = i"
							>
								<StarIcon
									class="h-7 w-7 transition-colors"
									:class="i <= formRating ? 'text-yellow-400' : 'text-button-text hover:text-yellow-300'"
									aria-hidden="true"
								/>
							</button>
						</div>
					</div>

					<div class="flex flex-col gap-1">
						<label for="review-body" class="text-sm font-medium text-secondary">
							Review <span class="font-normal text-secondary">(optional)</span>
						</label>
						<textarea
							id="review-body"
							v-model="formBody"
							class="resize-y rounded-lg border border-button-border bg-surface-3 p-3 text-primary placeholder:text-secondary focus:outline-none focus:ring-2 focus:ring-brand"
							rows="4"
							maxlength="8192"
							placeholder="Share your experience with this project..."
						/>
					</div>

					<div class="flex gap-2">
						<ButtonStyled color="brand" :disabled="formRating === 0 || submitting">
							<button type="button" @click="submitReview">
								<span v-if="submitting">Saving...</span>
								<span v-else>{{ ownReview ? 'Save changes' : 'Submit review' }}</span>
							</button>
						</ButtonStyled>
						<ButtonStyled v-if="ownReview" color="red" :disabled="deleting">
							<button type="button" @click="deleteReview">
								<span v-if="deleting">Deleting...</span>
								<span v-else>Delete review</span>
							</button>
						</ButtonStyled>
						<ButtonStyled v-if="!ownReview" type="outlined" @click="showForm = false">
							<button type="button">Cancel</button>
						</ButtonStyled>
					</div>
					<p v-if="formError" class="m-0 text-sm text-red">{{ formError }}</p>
				</div>
			</template>
		</section>

		<div v-if="reviewsLoading" class="flex justify-center py-8">
			<span class="text-secondary">Loading reviews...</span>
		</div>

		<template v-else-if="reviewsData && reviewsData.reviews.length > 0">
			<section
				v-for="review in reviewsData.reviews"
				:key="review.id"
				class="universal-card flex flex-col gap-3"
			>
				<div class="flex items-start justify-between gap-3">
					<div class="flex items-center gap-2">
						<span class="font-semibold text-primary">{{ review.user_id }}</span>
						<span class="text-sm text-secondary">&middot;</span>
						<time
							:datetime="review.created"
							class="text-sm text-secondary"
							:title="new Date(review.created).toLocaleString()"
						>
							{{ formatRelativeDate(review.created) }}
						</time>
					</div>
					<div class="flex shrink-0 gap-0.5">
						<StarIcon
							v-for="i in 5"
							:key="i"
							class="h-4 w-4"
							:class="i <= review.rating ? 'text-yellow-400' : 'text-secondary'"
							aria-hidden="true"
						/>
					</div>
				</div>
				<p v-if="review.body" class="m-0 whitespace-pre-wrap text-primary">{{ review.body }}</p>
			</section>
		</template>

		<div
			v-else-if="reviewsData && reviewsData.total === 0 && !ownReview"
			class="universal-card flex flex-col items-center gap-3 py-12 text-center"
		>
			<StarIcon class="h-10 w-10 text-secondary" aria-hidden="true" />
			<p class="m-0 text-secondary">No reviews yet. Be the first to leave one!</p>
		</div>

		<div
			v-if="reviewsData && reviewsData.total > reviewsData.reviews.length"
			class="flex justify-center"
		>
			<ButtonStyled type="outlined" @click="loadMore">
				<button type="button" :disabled="reviewsLoading">Load more</button>
			</ButtonStyled>
		</div>
	</div>
</template>
<script setup>
import { StarIcon } from '@modrinth/assets'
import { ButtonStyled, injectAuth, injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

const props = defineProps({
	project: {
		type: Object,
		default: () => ({}),
	},
})

const auth = injectAuth()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const showForm = ref(false)
const formRating = ref(0)
const formBody = ref('')
const formError = ref('')
const submitting = ref(false)
const deleting = ref(false)
const offset = ref(0)
const PAGE_SIZE = 20

const projectId = computed(() => props.project?.id)

const { data: reviewsData, isLoading: reviewsLoading } = useQuery({
	queryKey: computed(() => ['reviews', projectId.value, offset.value]),
	queryFn: () =>
		client.labrinth.reviews_v3.list(projectId.value, {
			count: PAGE_SIZE,
			offset: offset.value,
		}),
	enabled: computed(() => !!projectId.value),
})

const { data: ownReviewData } = useQuery({
	queryKey: computed(() => ['reviews', 'own', projectId.value]),
	queryFn: () => client.labrinth.reviews_v3.getOwnForProject(projectId.value),
	enabled: computed(() => !!projectId.value && !!auth?.user),
	retry: false,
})

const ownReview = computed(() => ownReviewData.value ?? null)

watch(
	ownReview,
	(review) => {
		if (review) {
			formRating.value = review.rating
			formBody.value = review.body
		}
	},
	{ immediate: true },
)

const averageRating = computed(() => {
	const reviews = reviewsData.value?.reviews
	if (!reviews || reviews.length === 0) return null
	const sum = reviews.reduce((acc, r) => acc + r.rating, 0)
	return sum / reviews.length
})

function formatRelativeDate(dateStr) {
	const date = new Date(dateStr)
	const now = new Date()
	const diffDays = Math.floor((now - date) / (1000 * 60 * 60 * 24))
	if (diffDays === 0) return 'today'
	if (diffDays === 1) return 'yesterday'
	if (diffDays < 30) return `${diffDays} days ago`
	if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`
	return `${Math.floor(diffDays / 365)} years ago`
}

async function submitReview() {
	if (formRating.value === 0) return
	submitting.value = true
	formError.value = ''

	try {
		if (ownReview.value) {
			await client.labrinth.reviews_v3.edit(ownReview.value.id, {
				rating: formRating.value,
				body: formBody.value,
			})
		} else {
			await client.labrinth.reviews_v3.create({
				project_id: projectId.value,
				rating: formRating.value,
				body: formBody.value,
			})
			showForm.value = false
		}

		await queryClient.invalidateQueries({ queryKey: ['reviews', projectId.value] })
		await queryClient.invalidateQueries({ queryKey: ['reviews', 'own', projectId.value] })
	} catch (err) {
		formError.value = err?.data?.description ?? 'An error occurred. Please try again.'
	} finally {
		submitting.value = false
	}
}

async function deleteReview() {
	if (!ownReview.value) return
	deleting.value = true

	try {
		await client.labrinth.reviews_v3.delete(ownReview.value.id)
		formRating.value = 0
		formBody.value = ''
		await queryClient.invalidateQueries({ queryKey: ['reviews', projectId.value] })
		await queryClient.invalidateQueries({ queryKey: ['reviews', 'own', projectId.value] })
	} catch {
		// silent failure - user can retry
	} finally {
		deleting.value = false
	}
}

function loadMore() {
	offset.value += PAGE_SIZE
}
</script>
