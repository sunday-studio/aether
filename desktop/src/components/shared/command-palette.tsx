import {
	CommandDialog,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from "cmdk";
import { Bookmark, Egg, Goal, ListTodo, Tag } from "lucide-react";
import * as React from "react";
import { useNavigate } from "react-router";
import { useSearch } from "~/aether-sdk";

interface CommandPaletteProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}

export const CommandPalette = ({ open, onOpenChange }: CommandPaletteProps) => {
	const navigate = useNavigate();
	const [searchQuery, setSearchQuery] = React.useState("");

	const { data: searchResults, isLoading } = useSearch(
		{
			q: searchQuery,
			limit: 20,
		},
		{
			query: {
				enabled: searchQuery.length > 0,
			},
		},
	);

	const handleSelect = (result: { type: string; id: string }) => {
		onOpenChange(false);
		setSearchQuery("");

		switch (result.type) {
			case "Entry":
				navigate(`/`);
				// TODO: Navigate to specific entry when entry detail view is available
				break;
			case "Task":
				navigate(`/tasks`);
				// TODO: Navigate to specific task when task detail view is available
				break;
			case "SubTask":
				navigate(`/tasks`);
				// TODO: Navigate to parent task with subtask highlighted
				break;
			case "Goal":
				navigate(`/tasks/goal/${result.id}`);
				break;
			case "Tag":
				navigate(`/`);
				// TODO: Filter by tag when tag filtering is available
				break;
			case "Bookmark":
				navigate(`/bookmarks`);
				// TODO: Navigate to specific bookmark when bookmark detail view is available
				break;
		}
	};

	const getIcon = (type: string) => {
		switch (type) {
			case "Entry":
				return <Egg className="size-4" />;
			case "Task":
			case "SubTask":
				return <ListTodo className="size-4" />;
			case "Goal":
				return <Goal className="size-4" />;
			case "Tag":
				return <Tag className="size-4" />;
			case "Bookmark":
				return <Bookmark className="size-4" />;
			default:
				return null;
		}
	};

	const getTitle = (result: {
		type: string;
		entry?: { document: string };
		task?: { title: string };
		subtask?: { title: string };
		goal?: { name: string };
		tag?: { name: string };
		bookmark?: { title?: string; url?: string };
	}) => {
		switch (result.type) {
			case "Entry":
				// Extract text from Lexical document JSON
				try {
					const doc = JSON.parse(result.entry?.document || "{}");
					const root = doc.root;
					if (root?.children?.[0]?.children?.[0]?.text) {
						return root.children[0].children[0].text;
					}
				} catch {
					// Fallback if parsing fails
				}
				return "Untitled Entry";
			case "Task":
				return result.task?.title || "Untitled Task";
			case "SubTask":
				return result.subtask?.title || "Untitled SubTask";
			case "Goal":
				return result.goal?.name || "Untitled Goal";
			case "Tag":
				return result.tag?.name || "Untitled Tag";
			case "Bookmark":
				return (
					result.bookmark?.title || result.bookmark?.url || "Untitled Bookmark"
				);
			default:
				return "Unknown";
		}
	};

	const results = searchResults?.data?.results || [];

	return (
		<CommandDialog open={open} onOpenChange={onOpenChange}>
			<CommandInput
				placeholder="Search resources..."
				value={searchQuery}
				onValueChange={setSearchQuery}
			/>
			<CommandList>
				{isLoading && searchQuery.length > 0 && (
					<div className="py-6 text-center text-sm command-palette-loading">
						Searching...
					</div>
				)}
				{!isLoading && searchQuery.length === 0 && (
					<CommandEmpty>Type to search...</CommandEmpty>
				)}
				{!isLoading && searchQuery.length > 0 && results.length === 0 && (
					<CommandEmpty>No results found.</CommandEmpty>
				)}
				{results.length > 0 && (
					<CommandGroup heading="Results">
						{results.map(
							(result: {
								type: string;
								id: string;
								entry?: { document: string; id: string };
								task?: { title: string; id: string };
								subtask?: { title: string; id: string };
								goal?: { name: string; id: string };
								tag?: { name: string; id: string };
								bookmark?: { title?: string; url?: string; id: string };
							}) => {
								// SearchResultResponse is a discriminated union with 'type' field
								const type = result.type;
								const id = result.id;

								return (
									<CommandItem
										key={`${type}-${id}`}
										value={`${type}-${id}`}
										onSelect={() => handleSelect({ type, id })}
									>
										<div className="flex items-center gap-2 w-full">
											{getIcon(type)}
											<span className="flex-1 truncate">
												{getTitle(result)}
											</span>
											<span className="text-xs capitalize command-palette-type-badge">
												{type.toLowerCase()}
											</span>
										</div>
									</CommandItem>
								);
							},
						)}
					</CommandGroup>
				)}
			</CommandList>
		</CommandDialog>
	);
};
