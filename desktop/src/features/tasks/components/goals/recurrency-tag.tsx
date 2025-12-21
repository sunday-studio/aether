export const RecurrencyTag = ({
	recurrenceType,
}: {
	recurrenceType: string;
}) => {
	return (
		<div className="rounded-lg px-1.5 h-6 bg-neutral-200/70 text-neutral-500 text-xs flex items-center justify-center">
			<span>{recurrenceType}</span>
		</div>
	);
};
