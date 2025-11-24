import type { DbEntry } from "~/aether-sdk/models";
import { Editor } from "~/components/editor/editor";

interface EntryEditorProps {
	document: DbEntry["document"];
	id: string;
	onChange: (document: string) => void;
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

export const EntryEditor = ({ document, id, onChange }: EntryEditorProps) => {
	if (!document) return <div className="bg-red-100">No data</div>;

	return (
		<div className="text-neutral-800 bg-transparent relative w-full">
			<Editor
				id={id}
				content={getEditorContent(document ?? "{}")}
				onChange={onChange}
				onHistoryChange={() => {}}
			/>
		</div>
	);
};
