import type { DbEntry } from "~/aether-sdk/models";
import { Editor } from "~/components/editor/editor";

interface EntryEditorProps {
	data: DbEntry;
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

export const EntryEditor = ({ data }: EntryEditorProps) => {
	if (!data) return <div className="bg-red-100">No data</div>;

	return (
		<div className="text-neutral-800 bg-transparent">
			<Editor
				id={data.createdAt ?? data.id ?? ""}
				content={getEditorContent(data.document ?? "{}")}
				onChange={() => {}}
				onHistoryChange={() => {}}
			/>
		</div>
	);
};
