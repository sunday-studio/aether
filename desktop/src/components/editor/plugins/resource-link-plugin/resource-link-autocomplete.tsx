import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import {
	LexicalTypeaheadMenuPlugin,
	MenuOption,
	useBasicTypeaheadTriggerMatch,
} from "@lexical/react/LexicalTypeaheadMenuPlugin";
import { useQuery } from "@tanstack/react-query";
import {
	$createTextNode,
	$getSelection,
	$isRangeSelection,
	type TextNode,
} from "lexical";
import { Bookmark, FileText, Goal, Link2, Square } from "lucide-react";
import { type JSX, useCallback, useMemo, useState } from "react";
import * as ReactDOM from "react-dom";
import { searchLinkableResources } from "~/aether-sdk";
import { $createResourceLinkNode } from "./resource-link-node";

interface LinkableResourceOptionParams {
	id: string;
	resourceType: string;
	title: string;
	preview?: string | null;
	onSelect: (id: string, resourceType: string, title: string) => void;
}

class LinkableResourceOption extends MenuOption {
	id: string;
	resourceType: string;
	title: string;
	preview: string | null;
	onSelect: (id: string, resourceType: string, title: string) => void;

	constructor(params: LinkableResourceOptionParams) {
		super(params.title);
		this.id = params.id;
		this.resourceType = params.resourceType;
		this.title = params.title;
		this.preview = params.preview || null;
		this.onSelect = params.onSelect.bind(this);
		this.key = `${params.resourceType}-${params.id}`;
	}
}

interface ResourceLinkMenuItem {
	index: number;
	isSelected: boolean;
	onClick: () => void;
	onMouseEnter: React.MouseEventHandler<HTMLLIElement>;
	option: LinkableResourceOption;
}

function ResourceLinkMenuItem({
	index,
	isSelected,
	onClick,
	onMouseEnter,
	option,
}: ResourceLinkMenuItem) {
	const getIcon = () => {
		switch (option.resourceType) {
			case "entry":
				return <FileText size={16} />;
			case "task":
				return <Square size={16} />;
			case "goal":
				return <Goal size={16} />;
			case "canvas":
				return <Square size={16} />;
			case "bookmark":
				return <Bookmark size={16} />;
			default:
				return <Link2 size={16} />;
		}
	};

	return (
		<li
			key={option.key}
			tabIndex={-1}
			role="option"
			id={`resource-link-item-${index}`}
			aria-selected={isSelected}
			onMouseEnter={onMouseEnter}
			onClick={onClick}
			className="flex cursor-pointer items-start justify-start p-2 rounded-lg gap-2 
      aria-[selected='true']:bg-stone-100 aria-[selected='true']:inset-ring aria-[selected='true']:inset-ring-stone-200
      dark:aria-[selected='true']:bg-stone-700 dark:aria-[selected='true']:inset-ring-0"
		>
			<div className="text-stone-500 w-5 h-5 flex items-center justify-center mt-0.5 flex-shrink-0">
				{getIcon()}
			</div>
			<div className="flex-1 min-w-0">
				<p className="text-stone-900 dark:text-stone-300 text-sm font-medium truncate">
					{option.title}
				</p>
				{option.preview && (
					<p className="text-stone-500 dark:text-stone-400 text-xs truncate mt-0.5">
						{option.preview}
					</p>
				)}
			</div>
		</li>
	);
}

export function ResourceLinkAutocomplete() {
	const [editor] = useLexicalComposerContext();
	const [queryString, setQueryString] = useState<string | null>(null);

	const checkForTriggerMatch = useBasicTypeaheadTriggerMatch("[[", {
		minLength: 0,
	});

	const { data: resources, isLoading } = useQuery({
		queryKey: ["searchLinkableResources", queryString || ""],
		queryFn: async () => {
			if (!queryString || queryString.trim().length === 0) {
				return { data: [] };
			}
			return searchLinkableResources({
				q: queryString,
				limit: 20,
			});
		},
		enabled: !!queryString && queryString.trim().length > 0,
	});

	const options = useMemo(() => {
		if (!resources?.data) return [];

		return resources.data.map(
			(resource) =>
				new LinkableResourceOption({
					id: resource.id,
					resourceType: resource.resourceType,
					title: resource.title,
					preview: resource.preview || null,
					onSelect: () => {
						// This is handled in onSelectOption
					},
				}),
		);
	}, [resources, editor]);

	const onSelectOption = useCallback(
		(
			selectedOption: LinkableResourceOption,
			nodeToRemove: TextNode | null,
			closeMenu: () => void,
			matchingString: string,
		) => {
			editor.update(() => {
				// Remove the trigger text ([[) and any query text
				if (nodeToRemove) {
					const textContent = nodeToRemove.getTextContent();
					// Remove [[ and everything after it
					const newText = textContent.replace(/\[\[.*$/, "");
					if (newText.length > 0) {
						nodeToRemove.setTextContent(newText);
					} else {
						nodeToRemove.remove();
					}
				}
				const selection = $getSelection();
				if ($isRangeSelection(selection)) {
					const linkNode = $createResourceLinkNode(
						selectedOption.resourceType,
						selectedOption.id,
						selectedOption.title,
					);
					selection.insertNodes([linkNode]);
				}
				closeMenu();
			});
		},
		[editor],
	);

	return (
		<LexicalTypeaheadMenuPlugin
			onQueryChange={setQueryString}
			onSelectOption={onSelectOption}
			triggerFn={checkForTriggerMatch}
			options={options}
			menuRenderFn={(
				anchorElementRef,
				{ selectedIndex, selectOptionAndCleanUp, setHighlightedIndex },
			) =>
				anchorElementRef.current
					? ReactDOM.createPortal(
							<div className="shadow-lg rounded-xl p-2 bg-white dark:bg-stone-800 w-[300px] max-h-[400px] overflow-y-auto">
								{isLoading ? (
									<div className="p-4 text-center text-stone-500 text-sm">
										Searching...
									</div>
								) : options.length === 0 && queryString ? (
									<div className="p-4 text-center text-stone-500 text-sm">
										No resources found
									</div>
								) : options.length > 0 ? (
									<ul className="list-none p-0 m-0">
										{options.map((option, i) => (
											<ResourceLinkMenuItem
												index={i}
												isSelected={selectedIndex === i}
												onClick={() => {
													setHighlightedIndex(i);
													selectOptionAndCleanUp(option);
												}}
												onMouseEnter={() => {
													setHighlightedIndex(i);
												}}
												key={option.key}
												option={option}
											/>
										))}
									</ul>
								) : null}
							</div>,
							anchorElementRef.current,
						)
					: null
			}
		/>
	);
}
