import { useQueryClient } from "@tanstack/react-query";
import { Check, X } from "lucide-react";
import { useState } from "react";
import { Button, DialogTrigger } from "react-aria-components";
import {
	getGetAllTagsQueryKey,
	useCreateTag,
	useGetAllTags,
} from "~/aether-sdk";
import { cn } from "~/utils/cn";
import { Popover } from "./popover";

type Tag = {
	id: string;
	name: string;
};

interface TagsPopoverSelectorProps {
	selectedTags: Tag[];
	onAddTag: (tagId: string) => void;
	onRemoveTag: (tagId: string) => void;
	onCreateTag: (tagId: string) => void;
	placeholder?: string;
	className?: string;
	triggerClassName?: string;
	customTrigger?: React.ReactNode;
}

export const popoverContentStyles = cn(
	"z-50 shadow-lg",
	"min-w-[16rem]",
	"origin-(--radix-popover-content-transform-origin)",
	"overflow-hidden",
	"rounded-xl",
	"bg-neutral-950 p-1 text-neutral-950",
	"data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
	"data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
	"data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
);

export const popoverItemStyles = cn(
	"relative flex items-center gap-2 text-neutral-300 w-full cursor-pointer justify-between",
	"rounded-lg px-2 py-1.5 text-sm",
	"cursor-default outline-hidden select-none",
	"focus:bg-neutral-00 hover:bg-neutral-800/90",
	"data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
);

export const searchInputStyles = cn(
	"w-full rounded-lg bg-neutral-800  border-neutral-700 px-3 py-2 text-sm outline-none text-neutral-200",
	"focus:border-neutral-600 placeholder:text-neutral-500",
);

export function TagsPopoverSelector(props: TagsPopoverSelectorProps) {
	const {
		selectedTags,
		onAddTag,
		onRemoveTag,
		onCreateTag,
		placeholder = "Search or create tag...",
		className,
		triggerClassName,
		customTrigger,
	} = props;

	const tagsQueryKey = getGetAllTagsQueryKey();
	const queryClient = useQueryClient();

	const { data: tagsResponse } = useGetAllTags();
	const { mutate: createTag } = useCreateTag();

	const [searchValue, setSearchValue] = useState("");

	// SDK now returns properly typed PaginatedTags
	const allTags: Tag[] = (tagsResponse?.status === 200 
		? tagsResponse.data?.items ?? [] 
		: []) as Tag[];

	// Filtering & creation logic
	const filteredTags = allTags.filter((tag: Tag) => {
		const tagName = tag.name?.toLowerCase() ?? "";
		const search = searchValue.toLowerCase();
		return tagName.includes(search);
	});

	const showCreateOption =
		!!onCreateTag &&
		searchValue.trim().length > 0 &&
		!allTags.some(
			(tag: Tag) => tag.name?.toLowerCase() === searchValue.toLowerCase(),
		);

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
					onCreateTag(data?.id ?? "");
					setSearchValue("");
				},
			},
		);
	};

	const hasTags = selectedTags.length > 0;

	return (
		<div className={cn("", className)}>
			<div className="flex flex-wrap gap-1 items-end justify-end">
				<DialogTrigger>
					<Button>
						{customTrigger ? (
							customTrigger
						) : hasTags ? (
							<div
								className={cn(
									"flex flex-col gap-1 items-end justify-end",
									triggerClassName,
								)}
							>
								{selectedTags.map((tag: Tag) => (
									<div
										key={tag.id}
										className="
                      flex items-center justify-between text-neutral-100 
                      text-xs p-1 px-2 rounded-full 
                      bg-linear-to-b from-green-800 to-green-900
                      gap-1 cursor-pointer"
									>
										<span>{tag.name}</span>
										<button
											type="button"
											onClick={(e) => {
												e.preventDefault();
												e.stopPropagation();
												onRemoveTag(tag.id);
											}}
											className="hover:bg-green-800 rounded-full hover:text-green-100 text-green-100 transition-colors duration-200"
										>
											<X className="size-3" />
										</button>
									</div>
								))}
							</div>
						) : (
							<button
								type="button"
								className={cn(
									"rounded-md  p-1 text-sm newsreader-font text-neutral-500",
									"hover:bg-neutral-100 hover:text-neutral-700",
									"transition-colors",
								)}
							>
								Add tag
							</button>
						)}
					</Button>
					<Popover className={popoverContentStyles}>
						<div
							// To focus input on open (manual since our Popover doesn't do this directly)
							tabIndex={-1}
							onAnimationEnd={() => {
								// Slight delay to ensure DOM is ready
								setTimeout(() => {
									const el = document.activeElement as HTMLElement;
									if (el) {
										const input = el.querySelector("input");
										if (input) input.focus();
									}
								}, 0);
							}}
						>
							<div className="sticky top-0 pb-1">
								<input
									type="text"
									placeholder={placeholder}
									value={searchValue}
									onChange={(e) => setSearchValue(e.target.value)}
									className={cn(searchInputStyles)}
								/>
							</div>
							<div className="max-h-48 overflow-y-auto">
								{showCreateOption && (
									<button
										type="button"
										onClick={() => {
											handleCreateTag(searchValue.trim());
											setSearchValue("");
										}}
										className={popoverItemStyles}
									>
										Create "{searchValue.trim()}"
									</button>
								)}

								<ul>
									{filteredTags.map((tag: Tag) => {
										const isAlreadyAdded = selectedTags.some(
											(t) => t.id === tag.id,
										);
										return (
											<li
												onKeyDown={(e) => {
													if (e.key === "Enter" || e.key === " ") {
														e.preventDefault();
														e.stopPropagation();
														if (!tag.id) return;
														onAddTag(tag.id!);
														setSearchValue("");
													}
												}}
												key={tag.id}
												onClick={() => {
													if (!tag.id) return;

													if (isAlreadyAdded) {
														onRemoveTag(tag.id);
													} else {
														onAddTag(tag.id);
														setSearchValue("");
													}
												}}
												className={popoverItemStyles}
											>
												<span>{tag.name}</span>
												<CheckboxItem isChecked={isAlreadyAdded} />
											</li>
										);
									})}
								</ul>
							</div>
						</div>
					</Popover>
				</DialogTrigger>
			</div>
		</div>
	);
}

const CheckboxItem = ({ isChecked }: { isChecked: boolean }) => {
	return (
		<span
			className={cn(
				"size-4 rounded-md bg-neutral-600 text-neutral-400 flex items-center justify-center bg-linear-to-b inset-shadow-xs",
				{
					" text-green-100 from-green-700 to-green-950  inset-shadow-green-700":
						isChecked,
					" from-neutral-600 to-neutral-700 text-white inset-shadow-neutral-700":
						!isChecked,
				},
			)}
		>
			{isChecked ? <Check className="size-3" /> : null}
		</span>
	);
};
