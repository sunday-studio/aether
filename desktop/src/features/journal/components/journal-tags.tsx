/** biome-ignore-all lint/style/noNonNullAssertion: some complaints; will look into it later */
import * as Popover from "@radix-ui/react-popover";
import { useQueryClient } from "@tanstack/react-query";
import { Check, X } from "lucide-react";
import { useEffect, useState } from "react";
import {
	// type AddTagsToEntryMutationBody,
	getGetAllTagsQueryKey,
	getGetEntriesQueryKey,
	useAddTagsToEntry,
	useCreateTag,
	useGetAllTags,
	useRemoveTagsFromEntry,
	// getGetEntryQueryKey,
	// useGetTags,
	// usePostEntryIdTags,
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
	const { data: tagsResponse } = useGetAllTags();

	const allTags: DbTag[] = (
		tagsResponse?.status === 200 ? tagsResponse.data : []
	) as DbTag[];

	const [entryTags, setEntryTags] = useState<DbTag[]>(entry.tags ?? []);

	const tagsQueryKey = getGetAllTagsQueryKey();
	const entriesQueryKey = getGetEntriesQueryKey();

	const [isOpen, setIsOpen] = useState(false);
	const [searchValue, setSearchValue] = useState("");

	// Automatically open popover when there are no tags
	useEffect(() => {
		if (entryTags.length === 0) {
			setIsOpen(true);
		}
	}, [entryTags.length]);

	const { mutate: addTagsToEntry } = useAddTagsToEntry();
	const { mutate: createTag } = useCreateTag();
	const { mutate: removeTagsFromEntry } = useRemoveTagsFromEntry();

	const handleAddTag = async (tagId: string) => {
		if (!entry.id) return;

		addTagsToEntry(
			{
				id: entry.id,
				data: [tagId],
			},
			{
				onSuccess: ({ data }) => {
					queryClient.invalidateQueries({ queryKey: entriesQueryKey });
					setEntryTags([...((data?.tags as unknown as DbTag[]) ?? [])]);
				},
			},
		);
	};

	const handleCreateTag = async (tagName: string) => {
		createTag(
			{
				data: {
					name: tagName.toLocaleLowerCase(),
				},
			},
			{
				onSuccess: ({ data }) => {
					queryClient.invalidateQueries({ queryKey: tagsQueryKey });
					handleAddTag(data.id!);
					setSearchValue("");
				},
			},
		);
	};

	const handleRemoveTag = (tagId: string) => {
		if (!entry.id) return;

		removeTagsFromEntry(
			{
				id: entry.id,
				data: tagId,
			},
			{
				onSuccess: () => {
					queryClient.invalidateQueries({ queryKey: entriesQueryKey });
					setEntryTags(entryTags.filter((t: DbTag) => t.id !== tagId));
				},
			},
		);
	};

	const filteredTags = allTags.filter((tag: DbTag) => {
		const tagName = tag.name?.toLowerCase() ?? "";
		const search = searchValue.toLowerCase();
		return tagName.includes(search);
	});

	const showCreateOption =
		searchValue.trim().length > 0 &&
		!allTags.some(
			(tag: DbTag) => tag.name?.toLowerCase() === searchValue.toLowerCase(),
		);

	const hasTags = entryTags.length > 0;

	return (
		<div className="mb-3">
			<div className="flex flex-wrap gap-1 items-end justify-end">
				<Popover.Root open={isOpen} onOpenChange={setIsOpen}>
					<Popover.Trigger asChild>
						{hasTags ? (
							<div className="flex flex-wrap gap-1 items-end justify-end">
								{entryTags.map((tag) => (
									<div
										key={tag.id}
										className="
						flex items-center justify-between bg-green-900 text-sky-100 
						text-xs p-1 px-2 rounded-full inset-ring-green-800 inset-ring-2
						gap-1 cursor-pointer"
									>
										<span>{tag.name}</span>
										<button
											type="button"
											onClick={(e) => {
												e.preventDefault();
												e.stopPropagation();
												handleRemoveTag(tag.id!);
											}}
											className="hover:bg-green-800 rounded-full hover:text-green-100 text-green-100 transition-colors duration-200"
										>
											<X className="size-3 " />
										</button>
									</div>
								))}
							</div>
						) : (
							<button
								type="button"
								className={cn(
									"rounded-md h-[24px] p-1 text-sm newsreader-font text-neutral-500",
									"hover:bg-neutral-100 hover:text-neutral-700",
									"transition-colors",
								)}
							>
								Add tag
							</button>
						)}
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
										onClick={() => handleCreateTag(searchValue.trim())}
										className={popoverItemStyles}
									>
										Create "{searchValue.trim()}"
									</button>
								)}
								{filteredTags.map((tag: DbTag) => {
									const isAlreadyAdded = entryTags.some(
										(t: DbTag) => t.id === tag.id,
									);

									return (
										<button
											key={tag.id}
											type="button"
											onClick={() => {
												if (!tag.id) return;

												if (isAlreadyAdded) {
													handleRemoveTag(tag.id);
												} else {
													handleAddTag(tag.id);
												}
											}}
											className={cn(popoverItemStyles, isAlreadyAdded && "")}
										>
											{tag.name}

											{isAlreadyAdded && (
												<Check className="size-3 ml-auto text-emerald-500" />
											)}
										</button>
									);
								})}
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
