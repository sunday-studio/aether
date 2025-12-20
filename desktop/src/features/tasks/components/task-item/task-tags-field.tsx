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

const TagItem = ({ label }: { label: string }) => {
	return (
		<div
			className={cn(
				"rounded-lg px-1.5 h-6",
				"bg-neutral-200/70 text-neutral-500",
				"flex items-center justify-center",
			)}
		>
			<span className={cn("text-shadow-3xs", "text-xs")}>{label}</span>
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

	return (
		<div className={cn("flex", "items-start", "shrink-0", "justify-start")}>
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
