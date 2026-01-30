import type { EntryWithTags } from "~/types/models";
import { Editor } from "~/components/editor/editor";
import { cn } from "~/utils/cn";

interface JournalEditorProps {
	document: EntryWithTags["document"];
	id: string;
	onChange: (document: string) => void;
	isSelected?: boolean;
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

export const JournalEditor = ({
	document,
	id,
	onChange,
	isSelected = false,
}: JournalEditorProps) => {
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
		</div>
	);
};
