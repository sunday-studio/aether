import { Editor } from '~/components/editor/editor';
import type { EntryWithTags } from '~/types/models';
import { cn } from '~/utils/cn';

interface JournalEditorProps {
	document: EntryWithTags['document'];
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
	if (!document) return <div className='bg-red-100'>No data</div>;

	return (
		<div
			className={cn(
				'relative -mx-3 -my-2 flex w-full gap-2 rounded-md bg-transparent px-3 py-1 text-neutral-800 transition-colors duration-150',
				'journal-editor-lines',
				isSelected &&
					'bg-green-50/50 text-green-700! ring ring-green-100 [&_.editor-root]:text-green-700!',
			)}
		>
			<Editor
				id={id}
				content={getEditorContent(document ?? '{}')}
				onChange={onChange}
				onHistoryChange={() => {}}
			/>
		</div>
	);
};
