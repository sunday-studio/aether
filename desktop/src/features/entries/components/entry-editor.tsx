import { format, formatDistanceToNow } from "date-fns";
import type { DbEntry } from "~/aether-sdk/models";
import { Editor } from "~/components/editor/editor";
import { cn } from "~/utils/cn";

interface EntryEditorProps {
	document: DbEntry["document"];
	id: string;
	onChange: (document: string) => void;
	createdAt: DbEntry["createdAt"];
	updatedAt: DbEntry["updatedAt"];
	isSelected: boolean;
}

export function getEditorContent(content: string) {
	try {
		const parsedContent = JSON.parse(content);

		if (parsedContent?.root?.children?.length > 0) {
			return content;
		}

		return null;
	} catch (error) {
		return null;
	}
}

export const EntryEditor = ({
	document,
	id,
	onChange,
	createdAt,
	updatedAt,
	isSelected = false,
}: EntryEditorProps) => {
	if (!document) return <div className="bg-red-100">No data</div>;

	return (
		<div
			className={cn(
				"text-neutral-800 bg-transparent relative w-full flex gap-2 px-3 -mx-3 py-2 -my-2 transition-colors duration-150 rounded-md",
				isSelected && "bg-blue-50/50 ring ring-blue-100",
			)}
		>
			<Editor
				id={id}
				content={getEditorContent(document ?? "{}")}
				onChange={onChange}
				onHistoryChange={() => {}}
			/>

			<div className="relative group w-fit ml-auto shrink-0">
				<p className="text-xs text-neutral-500 text-right newsreader-font px-1 py-0.5 rounded-md cursor-default">
					{formatDistanceToNow(new Date(updatedAt ?? ""), { addSuffix: true })}
				</p>

				{/* {createdAt && (
					<div className="absolute right-0 top-full mt-0 z-10 whitespace-nowrap text-xs bg-neutral-800 text-neutral-100 px-2 py-1 rounded-lg inset-shadow-2xs shadow opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity duration-150">
						created at {format(new Date(createdAt), "MMMM d, yyyy")}
					</div>
				)} */}
			</div>
		</div>
	);
};
