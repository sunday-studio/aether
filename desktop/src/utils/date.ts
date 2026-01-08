import { type CalendarDate, parseDate } from "@internationalized/date";

const SQLITE_NULL_DATE = "0001-01-01T00:00:00Z";

export const getDateValue = (value: string | undefined | Date) => {
	if (SQLITE_NULL_DATE === value || !value) {
		return undefined;
	}

	if (value instanceof Date) {
		return parseDate(value.toISOString()?.split("T")[0]);
	}

    console.log('value (getDateValue) ->', value)

	return undefined;
	

	const dateonly = value?.split("T")[0];
	return parseDate(dateonly as string);
};

/**
 * convert calendar date to iso string
 * @param date - calendar date
 * @returns iso string
 */
export const convertCalendarDateToIsoString = (date: CalendarDate) => {
	const now = new Date();
	const tempDate = date.toDate("UTC");
	tempDate.setHours(
		now.getHours(),
		now.getMinutes(),
		now.getSeconds(),
		now.getMilliseconds(),
	);
	return tempDate.toISOString();
};
