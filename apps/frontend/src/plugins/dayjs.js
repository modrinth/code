import dayjs from "dayjs";
import quarterOfYear from "dayjs/plugin/quarterOfYear";
import advanced from "dayjs/plugin/advancedFormat";
import relativeTime from "dayjs/plugin/relativeTime";

dayjs.extend(quarterOfYear);
dayjs.extend(advanced);
dayjs.extend(relativeTime);

export default defineNuxtPlugin(() => {
  return {
    provide: {
      dayjs,
    },
  };
});
