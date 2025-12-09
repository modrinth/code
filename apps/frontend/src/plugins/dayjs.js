import dayjs from 'dayjs'
import advanced from 'dayjs/plugin/advancedFormat'
import quarterOfYear from 'dayjs/plugin/quarterOfYear'
import relativeTime from 'dayjs/plugin/relativeTime'
import utc from 'dayjs/plugin/utc'

dayjs.extend(quarterOfYear)
dayjs.extend(advanced)
dayjs.extend(utc)
dayjs.extend(relativeTime)

export default defineNuxtPlugin(() => {
	return {
		provide: {
			dayjs,
		},
	}
})
