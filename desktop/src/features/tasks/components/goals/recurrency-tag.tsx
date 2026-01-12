export const RecurrencyTag = ({
	recurrenceType,
}: {
	recurrenceType: string;
}) => {
	const isNonRecurring = recurrenceType === "";
	return (
		<div className="rounded-lg px-1.5 h-6 bg-neutral-200/70 text-neutral-500 text-xs flex items-center justify-center">
			<span>{isNonRecurring ? "Non-recurring" : recurrenceType}</span>
		</div>
	);
};
