/** biome-ignore-all lint/suspicious/noArrayIndexKey: <explanation> */
import { cn } from "~/utils/cn";

interface AddNewButtonProps {
	onClick: () => void;
	shortcuts?: string[];
	label: string;
}

export const AddNewButton = ({
	onClick,
	shortcuts,
	label,
}: AddNewButtonProps) => {
	return (
		<button
			className={cn(
				"ring ring-neutral-200 text-neutral-700 flex items-center gap-1",
				"px-3 py-1.5 text-sm rounded-full bg-neutral-100",
				"hover:ring-neutral-300",
				"ring-3 transition-all duration-200 cursor-pointer",
			)}
			type="button"
			onClick={onClick}
		>
			<p className="text-sm">{label}</p>
			<div className="flex items-center justify-center gap-0.5">
				{shortcuts?.map(
					(shortcut, idx) =>
						shortcut && (
							<kbd
								key={`${shortcut}-${idx}`}
								className="px-1 bg-linear-to-b from-neutral-200 to-neutral-300 h-5 w-fit min-w-5 rounded-md text-xs font-medium pointer-events-none  inline-flex items-center justify-center gap-1 text-neutral-700 text-center select-none"
							>
								{shortcut}
							</kbd>
						),
				)}
			</div>
		</button>
	);
};
