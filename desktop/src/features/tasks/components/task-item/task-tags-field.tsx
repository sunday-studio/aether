import * as Popover from "@radix-ui/react-popover";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
	getGetInboxTasksQueryKey,
	useAddTagsToTask,
	useGetAllTags,
	useRemoveTagsFromTask,
} from "~/aether-sdk";
import type { DbTag } from "~/aether-sdk/models";
import { TagsPopoverSelector } from "~/components/shared/tags-popover-selector";

interface TaskTagsInputProps {
	value: DbTag[] | undefined;
	onChange: (value: string) => void;
	taskId: string;
}

export const TaskTagsInput = ({
	value,
	onChange,
	taskId,
}: TaskTagsInputProps) => {
	const { mutate: removeTagsFromTask } = useRemoveTagsFromTask();
	const { mutate: addTagsToTask } = useAddTagsToTask();
	// const hasTags = value?.length > 0;
	const [tags, setTags] = useState<DbTag[]>(value ?? []);

	const handleAddTag = (tag: string) => {
		// setTags([...tags, tag]);
		addTagsToTask(
			{
				id: taskId,
				data: [tag],
			},
			{
				onSuccess: ({ data }) => {
					// setTags([...tags, ...data!.tags!]);
				},
			},
		);
	};

	const handleRemoveTag = (tag: string) => {
		// setTags(tags.filter((t) => t !== tag));
		removeTagsFromTask({
			id: taskId,
			data: [tag],
		});
	};

	return (
		<div className="flex items-center shrink-0 w-[200px] overflow-hidden bg-red-100">
			<TagsPopoverSelector
				selectedTags={tags.map((tag) => ({
					id: tag.id!,
					name: tag.name!,
				}))}
				onAddTag={handleAddTag}
				onRemoveTag={handleRemoveTag}
				onCreateTag={() => {}}
				triggerClassName="flex-row bg-greeb-500 overflow-x-scroll w-full"
			/>
		</div>
	);
};
