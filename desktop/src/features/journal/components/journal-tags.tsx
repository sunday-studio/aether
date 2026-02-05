import { useQueryClient } from "@tanstack/react-query";
import { useAddTagsToEntry, useRemoveTagsFromEntry } from "~/aether-sdk";
import { AddTagsToEntity } from "~/components/shared/add-tag-to-entity";
import type { EntryWithTags } from "~/types/models";
import { cn } from "~/utils/cn";
import { invalidateEntryQueries } from "../invalidate-entry-queries";

export const popoverContentStyles = cn(
	"z-50 shadow-lg",
	"min-w-[16rem]",
	"origin-(--radix-popover-content-transform-origin)",
	"overflow-hidden",
	"rounded-lg",
	"bg-neutral-900 p-1 text-neutral-950",
	"data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
	"data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
	"data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
);

export const popoverItemStyles = cn(
	"relative flex items-center gap-2 text-neutral-200 w-full cursor-pointer",
	"rounded-md px-2 py-1.5 text-sm",
	"cursor-default outline-hidden select-none",
	"focus:bg-neutral-800 hover:bg-neutral-800",
	"data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
);

interface EntryTagsProps {
	entry: EntryWithTags;
}

export const EntryTags = ({ entry }: EntryTagsProps) => {
	const queryClient = useQueryClient();

	const { mutate: addTagsToEntry } = useAddTagsToEntry();
	const { mutate: removeTagsFromEntry } = useRemoveTagsFromEntry();

	const handleAddTag = async (tagId: string) => {
		if (!entry.id) return;

		addTagsToEntry(
			{
				id: entry.id,
				data: [tagId],
			},
			{
				onSuccess: () => invalidateEntryQueries(queryClient),
				onError: (error) => {
					console.error("Error adding tags to entry", error);
				},
			},
		);
	};

	const handleRemoveTag = (tagId: string) => {
		if (!entry.id) return;

		removeTagsFromEntry(
			{
				id: entry.id,
				data: [tagId],
			},
			{
				onSuccess: () => invalidateEntryQueries(queryClient),
			},
		);
	};

	return (
		<AddTagsToEntity
			selectedTags={entry.tags}
			entityId={entry.id}
			addTagToEntity={handleAddTag}
			removeTagFromEntity={handleRemoveTag}
		/>
	);
};
