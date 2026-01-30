/** biome-ignore-all lint/a11y/useFocusableInteractive: false positive */
/** biome-ignore-all lint/a11y/useSemanticElements: false positive */
import { Tag } from "lucide-react";
import { forwardRef, useMemo } from "react";
import { cn } from "tailwind-variants";
import { useAddTagsToTask, useRemoveTagsFromTask } from "~/aether-sdk";
import type { Tag as TagModel } from "~/aether-sdk/models";
import { TagsPopoverSelector } from "~/components/shared/tags-popover-selector";
import { Tooltip } from "~/components/shared/tooltip";
import { useOptimisticUpdateTaskQuery } from "../../use-optimistic-task-hooks";
import { TaskActionButton } from "./task-shared-components";

interface TaskTagsInputProps {
	value: TagModel[] | undefined;
	taskId: string;
}

const TagItem = ({ label }: { label: string }) => {
	return (
		<div
			className={cn(
				"rounded-lg px-1.5 h-6",
				"bg-neutral-200/70 text-neutral-500 text-xs",
				"flex items-center justify-center",
			)}
		>
			<span>{label}</span>
		</div>
	);
};

export const TaskTagsInput = ({ value, taskId }: TaskTagsInputProps) => {
	const { mutate: removeTagsFromTask } = useRemoveTagsFromTask();
	const { mutate: addTagsToTask } = useAddTagsToTask();

	const tags = value ?? [];
	const { updateLocalInstance, getPreviousData, queryClient, inboxTasksQueryKey, overdueTasksQueryKey } = useOptimisticUpdateTaskQuery();

	const handleAddTag = (tag: string) => {
		// Store previous state for rollback
		const previousData = getPreviousData();

		// Create optimistic tag
		const optimisticTag: TagModel = {
			id: `temp-${Date.now()}`,
			name: tag,
		};

		// Optimistically add the tag
		updateLocalInstance({
			id: taskId,
			data: {
				tags: [...tags, optimisticTag],
			},
		});

		addTagsToTask(
			{
				id: taskId,
				data: [tag],
			},
			{
				onSuccess: (response) => {
					// Replace optimistic data with real server response
					const taskData = response.data as unknown as { tags?: TagModel[] };
					updateLocalInstance({
						id: taskId,
						data: {
							tags: taskData?.tags ?? [],
						},
					});
				},
				onError: () => {
					// Rollback on error
					if (previousData.inbox) {
						queryClient.setQueryData(inboxTasksQueryKey, previousData.inbox);
					}
					if (previousData.overdue) {
						queryClient.setQueryData(overdueTasksQueryKey, previousData.overdue);
					}
				},
			},
		);
	};

	const handleRemoveTag = (tag: string) => {
		// Store previous state for rollback
		const previousData = getPreviousData();

		// Optimistically remove the tag
		updateLocalInstance({
			id: taskId,
			data: {
				tags: tags.filter((t) => t.name !== tag),
			},
		});

		removeTagsFromTask(
			{
				id: taskId,
				data: [tag],
			},
			{
				onSuccess: (response) => {
					// Confirm with real server response
					const taskData = response.data as unknown as { tags?: TagModel[] };
					updateLocalInstance({
						id: taskId,
						data: {
							tags: taskData?.tags ?? [],
						},
					});
				},
				onError: () => {
					// Rollback on error
					if (previousData.inbox) {
						queryClient.setQueryData(inboxTasksQueryKey, previousData.inbox);
					}
					if (previousData.overdue) {
						queryClient.setQueryData(overdueTasksQueryKey, previousData.overdue);
					}
				},
			},
		);
	};

	const CustomTrigger = forwardRef<
		HTMLDivElement,
		React.HTMLAttributes<HTMLDivElement>
	>(() => {
		const hasTags = tags.length > 0;
		const hasMoreThan3Tags = tags.length > 3;

		const tagsDisplayString = hasMoreThan3Tags
			? `${tags[0]?.name} & ${tags.length - 1} more`
			: undefined;

		if (!hasTags) {
			return (
				<TaskActionButton>
					<Tag size={14} strokeWidth={3} />
				</TaskActionButton>
			);
		}

		if (hasMoreThan3Tags) {
			return <TagItem label={tagsDisplayString ?? ""} />;
		}

		if (!hasMoreThan3Tags) {
			return (
				<div className="flex gap-1">
					{tags.map((tag) => (
						<TagItem key={tag.id} label={tag.name ?? ""} />
					))}
				</div>
			);
		}
	});

	const selectedTags = useMemo(() => {
		return tags.map((tag) => ({
			id: tag.id ?? "",
			name: tag.name ?? "",
		}));
	}, [tags]);

	return (
		<div className={cn("flex", "items-start", "shrink-0", "justify-start")}>
			<TagsPopoverSelector
				selectedTags={selectedTags}
				onAddTag={handleAddTag}
				onRemoveTag={handleRemoveTag}
				onCreateTag={() => {}}
				customTrigger={
					<Tooltip
						content="Add tags"
						trigger={<CustomTrigger />}
						disabled={Boolean(tags.length)}
					/>
				}
			/>
		</div>
	);
};
