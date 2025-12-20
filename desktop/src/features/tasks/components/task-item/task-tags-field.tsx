/** biome-ignore-all lint/a11y/useFocusableInteractive: false positive */
/** biome-ignore-all lint/a11y/useSemanticElements: false positive */
import { Tag } from "lucide-react";
import { forwardRef, useState } from "react";
import { cn } from "tailwind-variants";
import { useAddTagsToTask, useRemoveTagsFromTask } from "~/aether-sdk";
import type { DbTag } from "~/aether-sdk/models";
import { TagsPopoverSelector } from "~/components/shared/tags-popover-selector";

interface TaskTagsInputProps {
	value: DbTag[] | undefined;
	onChange: (value: string) => void;
	taskId: string;
}

const TagItem = ({ tag }: { tag: DbTag }) => {
	return (
		<div
			className={cn(
				"rounded-lg",
				"bg-linear-to-b from-neutral-100 to-neutral-200",
				"text-neutral-400 text-sm",
				"flex items-center justify-center",
				"px-1.5 h-6",
				"inset-ring-1 inset-ring-neutral-200",
			)}
		>
			<span className={cn("text-shadow-3xs", "text-xs")}>{tag.name}</span>
		</div>
	);
};

export const TaskTagsInput = ({
	value,
	onChange,
	taskId,
}: TaskTagsInputProps) => {
	const { mutate: removeTagsFromTask } = useRemoveTagsFromTask();
	const { mutate: addTagsToTask } = useAddTagsToTask();
	const [tags, setTags] = useState<DbTag[]>(value ?? []);

	const handleAddTag = (tag: string) => {
		addTagsToTask(
			{
				id: taskId,
				data: [tag],
			},
			{
				onSuccess: ({ data }) => {},
			},
		);
	};

	const handleRemoveTag = (tag: string) => {
		removeTagsFromTask({
			id: taskId,
			data: [tag],
		});
	};

	const CustomTrigger = forwardRef<
		HTMLDivElement,
		React.HTMLAttributes<HTMLDivElement>
	>((props, ref) => {
		const hasTags = tags.length > 0;

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
					tags.map((tag) => <TagItem key={tag.id} tag={tag} />)
				) : (
					<Tag size={14} strokeWidth={3} />
				)}
			</div>
		);
	});

	return (
		<div
			className={cn(
				"flex-1",
				"flex",
				"w-full",
				"items-start",
				"shrink-0",
				"justify-start",
			)}
		>
			<TagsPopoverSelector
				selectedTags={tags.map((tag) => ({
					id: tag.id ?? "",
					name: tag.name ?? "",
				}))}
				onAddTag={handleAddTag}
				onRemoveTag={handleRemoveTag}
				onCreateTag={() => {}}
				customTrigger={<CustomTrigger />}
			/>
		</div>
	);
};
