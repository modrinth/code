<script setup lang="ts">
const vintl = useVIntl()
const { formatMessage } = vintl

const messages = defineMessages({
  frogTitle: {
    id: 'frog.title',
    defaultMessage: 'Frog',
  },
  frogDescription: {
    id: 'frog',
    defaultMessage: "You've been frogged! 🐸",
  },
  frogAltText: {
    id: 'frog.altText',
    defaultMessage: 'A photorealistic painting of a frog labyrinth',
  },
  frogSinceOpened: {
    id: 'frog.sinceOpened',
    defaultMessage: 'This page was opened {ago}',
  },
  frogFroggedPeople: {
    id: 'frog.froggedPeople',
    defaultMessage:
      '{count, plural, one {{count} more person} other {{count} more people}} were also frogged!',
  },
})

const formatCompactNumber = useCompactNumber()

const formatRelativeTime = useRelativeTime()

const pageOpen = useState('frogPageOpen', () => Date.now())
const peopleFrogged = useState('frogPeopleFrogged', () => Math.round(Math.random() * 100_000_000))
const peopleFroggedCount = computed(() => formatCompactNumber(peopleFrogged.value))

let interval: ReturnType<typeof setTimeout>

const formattedOpenedCounter = ref(formatRelativeTime(Date.now()))

onMounted(() => {
  interval = setInterval(() => {
    formattedOpenedCounter.value = formatRelativeTime(pageOpen.value)
  }, 1000)
})

onUnmounted(() => clearInterval(interval))
</script>

<template>
  <div class="card">
    <h1>{{ formatMessage(messages.frogTitle) }}</h1>
    <p>{{ formatMessage(messages.frogDescription) }}</p>
    <img src="https://cdn.modrinth.com/frog.png" :alt="formatMessage(messages.frogAltText)" />
    <p>{{ formatMessage(messages.frogSinceOpened, { ago: formattedOpenedCounter }) }}</p>
    <p>{{ formatMessage(messages.frogFroggedPeople, { count: peopleFroggedCount }) }}</p>
  </div>
</template>

<style lang="scss" scoped>
.card {
  width: calc(100% - 2 * var(--spacing-card-md));
  max-width: 1280px;
  margin-inline: auto;
  text-align: center;
  box-sizing: border-box;
  margin-block: var(--spacing-card-md);
}

img {
  margin-block: 0 1.5rem;
  width: 60%;
  max-width: 40rem;
}
</style>
