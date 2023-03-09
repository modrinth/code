import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

// eslint-disable-next-line import/no-named-as-default-member
dayjs.extend(relativeTime)

export default defineNuxtPlugin(() => {
  return {
    provide: {
      dayjs,
    },
  }
})
