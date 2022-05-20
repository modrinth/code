/**
 * Human readable elapsed or remaining time (example: 3 minutes ago)
 * @author https://github.com/victornpb
 * @see https://stackoverflow.com/a/67338038/938822
 */
export function ago(
    /** A Date object, timestamp or string parsable with Date.parse() */
    date: string | number | Date,
    /** A Date object, timestamp or string parsable with Date.parse() */
    nowDate: string | number | Date = Date.now(),
    /** A Intl formater */
    rft: Intl.RelativeTimeFormat = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' })
): string {
    const SECOND = 1000;
    const MINUTE = 60 * SECOND;
    const HOUR = 60 * MINUTE;
    const DAY = 24 * HOUR;
    const WEEK = 7 * DAY;
    const MONTH = 30 * DAY;
    const YEAR = 365 * DAY;
    const intervals = [
        { ge: YEAR, divisor: YEAR, unit: 'year' },
        { ge: MONTH, divisor: MONTH, unit: 'month' },
        { ge: WEEK, divisor: WEEK, unit: 'week' },
        { ge: DAY, divisor: DAY, unit: 'day' },
        { ge: HOUR, divisor: HOUR, unit: 'hour' },
        { ge: MINUTE, divisor: MINUTE, unit: 'minute' },
        { ge: 30 * SECOND, divisor: SECOND, unit: 'seconds' },
        { ge: 0, divisor: 1, text: 'just now' },
    ];
    const now = typeof nowDate === 'object' ? nowDate.getTime() : new Date(nowDate).getTime();
    const diff = now - (typeof date === 'object' ? date : new Date(date)).getTime();
    const diffAbs = Math.abs(diff);
    for (const interval of intervals) {
        if (diffAbs >= interval.ge) {
            const x = Math.round(Math.abs(diff) / interval.divisor);
            const isFuture = diff < 0;
            return interval.unit
                ? rft.format(isFuture ? x : -x, interval.unit as Unit)
                : interval.text;
        }
    }
}

type Unit =
    | 'second'
    | 'seconds'
    | 'minute'
    | 'minutes'
    | 'hour'
    | 'hours'
    | 'day'
    | 'days'
    | 'week'
    | 'weeks'
    | 'month'
    | 'months'
    | 'year'
    | 'years';
