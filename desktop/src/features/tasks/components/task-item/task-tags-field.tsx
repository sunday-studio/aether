/** biome-ignore-all lint/a11y/useFocusableInteractive: false positive */
/** biome-ignore-all lint/a11y/useSemanticElements: false positive */
import { Tag } from "lucide-react";
import { forwardRef, useMemo, useState } from "react";
import { cn } from "tailwind-variants";
import { useAddTagsToTask, useRemoveTagsFromTask } from "~/aether-sdk";
import type { DbTag } from "~/aether-sdk/models";
import { TagsPopoverSelector } from "~/components/shared/tags-popover-selector";
import { useOptimisticUpdateTaskQuery } from "../../use-optimistic-update-task";

interface TaskTagsInputProps {
	value: DbTag[] | undefined;
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
	// const [tags, setTags] = useState<DbTag[]>(value ?? []);

	const tags = value ?? [];
	const { updateLocalInstance } = useOptimisticUpdateTaskQuery();

	const handleAddTag = (tag: string) => {
		addTagsToTask(
			{
				id: taskId,
				data: [tag],
			},
			{
				onSuccess: ({ data }) => {
					updateLocalInstance({
						id: taskId,
						data: {
							tags: (data?.tags as DbTag[]) ?? [],
						},
					});

					// setTags((data?.tags as DbTag[]) ?? []);
				},
			},
		);
	};

	const handleRemoveTag = (tag: string) => {
		removeTagsFromTask(
			{
				id: taskId,
				data: [tag],
			},
			{
				onSuccess: ({ data }) => {
					updateLocalInstance({
						id: taskId,
						data: {
							tags: (data?.tags as DbTag[]) ?? [],
						},
					});

					// setTags((data?.tags as DbTag[]) ?? []);
				},
			},
		);
	};

	const CustomTrigger = forwardRef<
		HTMLDivElement,
		React.HTMLAttributes<HTMLDivElement>
	>((props, ref) => {
		const hasTags = tags.length > 0;
		const hasMoreThan3Tags = tags.length > 3;

		const tagsDisplayString = hasMoreThan3Tags
			? `${tags[0]?.name} & ${tags.length - 1} more`
			: undefined;

		return (
			<div
				role="button"
				ref={ref}
				{...props}
				className={cn(
					hasTags
						? ["w-full", "shrink-0", "flex-1", "flex-wrap", "flex", "gap-0.5"]
						: [
								"w-6",
								"h-6",
								"bg-neutral-200",
								"text-neutral-400",
								"text-sm",
								"flex",
								"items-center",
								"justify-center",
								"rounded-lg",
							],
				)}
			>
				{hasTags ? (
					hasMoreThan3Tags ? (
						<TagItem label={tagsDisplayString ?? ""} />
					) : (
						tags.map((tag) => <TagItem key={tag.id} label={tag.name ?? ""} />)
					)
				) : (
					<Tag size={14} strokeWidth={3} />
				)}
			</div>
		);
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
				customTrigger={<CustomTrigger />}
			/>
		</div>
	);
};
