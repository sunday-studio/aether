import { Tag } from "lucide-react";
import { useMemo } from "react";
import { cn } from "tailwind-variants";
import type { Tag as TagModel } from "~/aether-sdk/models";
import { TagsPopoverSelector } from "~/components/shared/tags-popover-selector";
import { Tooltip } from "~/components/shared/tooltip";
import { TaskActionButton } from "~/features/tasks/components/task-item/task-shared-components";

interface AddTagsToEntityProps {
	selectedTags: TagModel[] | undefined;
	entityId: string;
	addTagToEntity: (tagId: string) => void;
	removeTagFromEntity: (tagId: string) => void;
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

function renderTriggerContent(tags: TagModel[]) {
	const hasTags = tags.length > 0;
	const hasMoreThan3Tags = tags.length > 3;

	if (!hasTags) {
		return (
			<TaskActionButton>
				<Tag size={14} strokeWidth={3} />
			</TaskActionButton>
		);
	}

	if (hasMoreThan3Tags) {
		const tagsDisplayString = `${tags[0]?.name} & ${tags.length - 1} more`;
		return <TagItem label={tagsDisplayString} />;
	}

	return (
		<div className="flex gap-1">
			{tags.map((tag) => (
				<TagItem key={tag.id} label={tag.name ?? ""} />
			))}
		</div>
	);
}

export const AddTagsToEntity = ({
	selectedTags,
	addTagToEntity,
	removeTagFromEntity,
}: AddTagsToEntityProps) => {
	const tags = selectedTags ?? [];

	const triggerContent = useMemo(() => renderTriggerContent(tags), [tags]);

	const tagsToDisplay = useMemo(() => {
		return tags.map((tag) => ({
			id: tag.id ?? "",
			name: tag.name ?? "",
		}));
	}, [tags]);

	return (
		<div className={cn("flex", "items-start", "shrink-0", "justify-start")}>
			<TagsPopoverSelector
				selectedTags={tagsToDisplay}
				onAddTag={addTagToEntity}
				onRemoveTag={removeTagFromEntity}
				onCreateTag={() => {}}
				customTrigger={
					<Tooltip
						content="Add tags"
						trigger={triggerContent}
						disabled={Boolean(tagsToDisplay.length)}
					/>
				}
			/>
		</div>
	);
};
