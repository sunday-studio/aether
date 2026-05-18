import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import {
	LexicalTypeaheadMenuPlugin,
	MenuOption,
	useBasicTypeaheadTriggerMatch,
} from '@lexical/react/LexicalTypeaheadMenuPlugin';
import { useQuery } from '@tanstack/react-query';
import { $getSelection, $isRangeSelection, type TextNode } from 'lexical';
import { FileText, Goal, Link2, Square } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import * as ReactDOM from 'react-dom';
import { searchLinkableResources } from '~/aether-sdk';
import { $createResourceLinkNode } from './resource-link-node';

const hiddenResourceTypes = new Set(['canvas', 'bookmark']);

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
			case 'entry':
				return <FileText size={16} />;
			case 'task':
				return <Square size={16} />;
			case 'goal':
				return <Goal size={16} />;
			default:
				return <Link2 size={16} />;
		}
	};

	return (
		<li
			key={option.key}
			tabIndex={-1}
			role='option'
			id={`resource-link-item-${index}`}
			aria-selected={isSelected}
			onMouseEnter={onMouseEnter}
			onClick={onClick}
			className="flex cursor-pointer items-start justify-start gap-2 rounded-lg p-2 aria-[selected='true']:bg-stone-100 aria-[selected='true']:inset-ring aria-[selected='true']:inset-ring-stone-200 dark:aria-[selected='true']:bg-stone-700 dark:aria-[selected='true']:inset-ring-0"
		>
			<div className='mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center text-stone-500'>
				{getIcon()}
			</div>
			<div className='min-w-0 flex-1'>
				<p className='truncate text-sm font-medium text-stone-900 dark:text-stone-300'>
					{option.title}
				</p>
				{option.preview && (
					<p className='mt-0.5 truncate text-xs text-stone-500 dark:text-stone-400'>
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

	const checkForTriggerMatch = useBasicTypeaheadTriggerMatch('[[', {
		minLength: 0,
	});

	const { data: resources, isLoading } = useQuery({
		queryKey: ['searchLinkableResources', queryString || ''],
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

		return resources.data
			.filter(resource => !hiddenResourceTypes.has(resource.resourceType))
			.map(
				resource =>
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
		) => {
			editor.update(() => {
				// Remove the trigger text ([[) and any query text
				if (nodeToRemove) {
					const textContent = nodeToRemove.getTextContent();
					// Remove [[ and everything after it
					const newText = textContent.replace(/\[\[.*$/, '');
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
							<div className='max-h-[400px] w-[300px] overflow-y-auto rounded-xl bg-white p-2 shadow-lg dark:bg-stone-800'>
								{isLoading ? (
									<div className='p-4 text-center text-sm text-stone-500'>Searching...</div>
								) : options.length === 0 && queryString ? (
									<div className='p-4 text-center text-sm text-stone-500'>No resources found</div>
								) : options.length > 0 ? (
									<ul className='m-0 list-none p-0'>
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
