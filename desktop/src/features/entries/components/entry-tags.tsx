import * as Popover from "@radix-ui/react-popover";
import { useQueryClient } from "@tanstack/react-query";
import { X } from "lucide-react";
import { useEffect, useState } from "react";
import {
	getGetEntryQueryKey,
	useGetTags,
	usePostEntryIdTags,
} from "~/aether-sdk";
import type { DbEntry, DbTag } from "~/aether-sdk/models";
import { cn } from "~/utils/cn";

const popoverContentStyles = cn(
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

const popoverItemStyles = cn(
	"relative flex items-center gap-2 text-neutral-200 w-full cursor-pointer",
	"rounded-md px-2 py-1.5 text-sm",
	"cursor-default outline-hidden select-none",
	"focus:bg-neutral-800 hover:bg-neutral-800",
	"data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
);

interface EntryTagsProps {
	entry: DbEntry;
}

export const EntryTags = ({ entry }: EntryTagsProps) => {
	const queryClient = useQueryClient();
	const { data: tagsResponse } = useGetTags();
	const allTags: DbTag[] = (
		tagsResponse?.status === 200 ? tagsResponse.data : []
	) as DbTag[];
	const entryTags: DbTag[] = entry.tags ?? [];

	const [isOpen, setIsOpen] = useState(false);
	const [searchValue, setSearchValue] = useState("");

	// Automatically open popover when there are no tags
	useEffect(() => {
		if (entryTags.length === 0) {
			setIsOpen(true);
		}
	}, [entryTags.length]);

	const { mutate: updateTags } = usePostEntryIdTags();

	const handleAddTag = (tagName: string) => {
		if (!entry.id) return;

		const existingTagNames = entryTags
			.map((t) => t.name)
			.filter(Boolean) as string[];
		const newTagNames = [...existingTagNames, tagName];

		updateTags(
			{
				id: entry.id,
				data: newTagNames,
			},
			{
				onSuccess: () => {
					queryClient.invalidateQueries({ queryKey: getGetEntryQueryKey() });
					setSearchValue("");
					// Don't close popover to allow multiple selections
				},
			},
		);
	};

	const handleRemoveTag = (tagToRemove: DbTag) => {
		if (!entry.id) return;

		const newTagNames = entryTags
			.filter((t) => t.id !== tagToRemove.id)
			.map((t) => t.name)
			.filter(Boolean) as string[];

		updateTags(
			{
				id: entry.id,
				data: newTagNames,
			},
			{
				onSuccess: () => {
					queryClient.invalidateQueries({ queryKey: getGetEntryQueryKey() });
				},
			},
		);
	};

	const filteredTags = allTags.filter((tag: DbTag) => {
		const tagName = tag.name?.toLowerCase() ?? "";
		const search = searchValue.toLowerCase();
		const isAlreadyAdded = entryTags.some((t: DbTag) => t.id === tag.id);
		return tagName.includes(search) && !isAlreadyAdded;
	});

	const showCreateOption =
		searchValue.trim().length > 0 &&
		!allTags.some(
			(tag: DbTag) => tag.name?.toLowerCase() === searchValue.toLowerCase(),
		);

	// Empty state - just show trigger that auto-opens
	if (entryTags.length === 0 && !isOpen) {
		return (
			<div className="mb-3">
				<Popover.Root open={isOpen} onOpenChange={setIsOpen}>
					<Popover.Trigger asChild>
						<button
							type="button"
							className="text-sm text-neutral-500 hover:text-neutral-700 transition-colors"
						>
							Add tags...
						</button>
					</Popover.Trigger>
				</Popover.Root>
			</div>
		);
	}

	return (
		<div className="mb-3">
			<div className="flex flex-wrap gap-2">
				{entryTags.map((tag) => (
					<div
						key={tag.id}
						className={cn(
							"flex items-center gap-1 rounded-md bg-neutral-100 px-2 py-1 text-sm",
						)}
					>
						<span>{tag.name}</span>
						<button
							type="button"
							onClick={() => handleRemoveTag(tag)}
							className={cn(
								"rounded-sm hover:bg-neutral-200",
								"transition-colors",
							)}
						>
							<X className="size-3" />
						</button>
					</div>
				))}

				<Popover.Root open={isOpen} onOpenChange={setIsOpen}>
					<Popover.Trigger asChild>
						<button
							type="button"
							className={cn(
								"rounded-md px-2 py-1 text-sm text-neutral-500",
								"hover:bg-neutral-100 hover:text-neutral-700",
								"transition-colors",
							)}
						>
							+ Add tag
						</button>
					</Popover.Trigger>
					<Popover.Portal>
						<Popover.Content
							className={popoverContentStyles}
							sideOffset={5}
							onOpenAutoFocus={(e) => {
								e.preventDefault();
								const content = e.currentTarget as HTMLElement;
								setTimeout(() => {
									const input = content?.querySelector("input");
									input?.focus();
								}, 0);
							}}
						>
							<div className="sticky top-0 bg-neutral-900 pb-1">
								<input
									type="text"
									placeholder="Search or create tag..."
									value={searchValue}
									onChange={(e) => setSearchValue(e.target.value)}
									onBlur={(e) => {
										// Don't close if clicking inside the popover
										if (
											e.currentTarget.parentElement?.contains(e.relatedTarget)
										) {
											return;
										}
									}}
									className={cn(
										"w-full rounded-md bg-neutral-800 border-neutral-700 px-3 py-2 text-sm outline-none text-neutral-200",
										"focus:border-neutral-600 placeholder:text-neutral-500",
									)}
								/>
							</div>
							<div className="max-h-48 overflow-y-auto">
								{showCreateOption && (
									<button
										type="button"
										onClick={() => handleAddTag(searchValue.trim())}
										className={popoverItemStyles}
									>
										Create "{searchValue.trim()}"
									</button>
								)}
								{filteredTags.map((tag: DbTag) => (
									<button
										key={tag.id}
										type="button"
										onClick={() => tag.name && handleAddTag(tag.name)}
										className={popoverItemStyles}
									>
										{tag.name}
									</button>
								))}
								{!showCreateOption && filteredTags.length === 0 && (
									<div className="px-2 py-1.5 text-sm text-neutral-500">
										No tags found
									</div>
								)}
							</div>
						</Popover.Content>
					</Popover.Portal>
				</Popover.Root>
			</div>
		</div>
	);
};
