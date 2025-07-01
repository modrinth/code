<script setup lang="ts">
import dayjs from 'dayjs'

export interface Article {
  path: string
  thumbnail: string
  title: string
  summary: string
  date: string
}

defineProps<{
  article: Article
}>()

const emit = defineEmits<{
  view: [article: Article]
}>()

const handleClick = (article: Article) => {
  emit('view', article)
}
</script>

<template>
  <div
    class="active:scale-[0.99]! group flex flex-col transition-all ease-in-out hover:brightness-125 cursor-pointer"
    @click="handleClick(article)"
  >
    <article class="flex h-full grow flex-col gap-4">
      <img
        :src="article.thumbnail"
        class="aspect-video w-full rounded-xl border-[1px] border-solid border-button-border object-cover"
      />
      <div class="flex grow flex-col gap-2">
        <h3 class="m-0 text-base leading-tight group-hover:underline">
          {{ article.title }}
        </h3>
        <p v-if="article.summary" class="m-0 text-sm leading-tight">
          {{ article.summary }}
        </p>
        <div class="mt-auto text-sm text-secondary">
          {{ dayjs(article.date).format('MMMM D, YYYY') }}
        </div>
      </div>
    </article>
  </div>
</template>
