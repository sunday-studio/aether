import { useEffect } from "react";
import { $isLinkNode } from "@lexical/link";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $findMatchingParent, isHTMLAnchorElement } from "@lexical/utils";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
	$getNearestNodeFromDOMNode,
	$getSelection,
	$isElementNode,
	$isRangeSelection,
	getNearestEditorFromDOMNode,
} from "lexical";

function findMatchingDOM(
	startNode: Node | null,
	predicate: (node: Node) => boolean,
): HTMLElement | null {
	if (!predicate) return null;
	let node: Node | null = startNode;
	while (node != null) {
		if (predicate(node)) {
			return node as HTMLElement;
		}
		node = (node as HTMLElement).parentNode;
	}
	return null;
}

export default function ClickableLinkPlugin() {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		function onClick(event: MouseEvent) {
			const target = event.target as Node;
			if (!(target instanceof Node)) {
				return;
			}
			const nearestEditor = getNearestEditorFromDOMNode(target);

			if (nearestEditor === null) {
				return;
			}

			let url: string | null = null;
			nearestEditor.update(() => {
				const clickedNode = $getNearestNodeFromDOMNode(target);
				if (clickedNode !== null) {
					const maybeLinkNode = $findMatchingParent(
						clickedNode,
						$isElementNode,
					);
					if ($isLinkNode(maybeLinkNode)) {
						url = maybeLinkNode.getURL();
					} else {
						const a = findMatchingDOM(
							target,
							isHTMLAnchorElement as (node: Node) => boolean,
						) as HTMLAnchorElement | null;
						if (a !== null) {
							url = a.href;
						}
					}
				}
			});

			if (!url) {
				return;
			}

			// Allow user to select link text without following url
			const selection = editor.getEditorState().read($getSelection);
			if ($isRangeSelection(selection) && !selection.isCollapsed()) {
				event.preventDefault();
				return;
			}
			openUrl(url);
			event.preventDefault();
		}

		function onMouseUp(event: MouseEvent) {
			if (event.button === 1 && editor.isEditable()) {
				onClick(event);
			}
		}

		return editor.registerRootListener((rootElement, prevRootElement) => {
			if (prevRootElement) {
				prevRootElement.removeEventListener("click", onClick);
				prevRootElement.removeEventListener("mouseup", onMouseUp);
			}
			if (rootElement) {
				rootElement.addEventListener("click", onClick);
				rootElement.addEventListener("mouseup", onMouseUp);
			}
		});
	}, [editor]);

	return null;
}
