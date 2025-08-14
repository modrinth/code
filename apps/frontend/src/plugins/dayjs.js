import dayjs from 'dayjs'
import advanced from 'dayjs/plugin/advancedFormat'
import quarterOfYear from 'dayjs/plugin/quarterOfYear'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(quarterOfYear)
dayjs.extend(advanced)
dayjs.extend(relativeTime)

export default defineNuxtPlugin(() => {
	return {
		provide: {
			dayjs,
		},
	}
})
