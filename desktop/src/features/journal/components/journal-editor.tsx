import { Editor } from "~/components/editor/editor";
import type { EntryWithTags } from "~/types/models";
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
				"text-neutral-800 bg-transparent relative w-full flex gap-2 px-3 -mx-3 py-1 -my-2 transition-colors duration-150 rounded-md",
				isSelected &&
					"bg-green-50/50 ring ring-green-100 text-green-700! &> .editor-root { color: var(--color-green-700) !important; }",
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
