import type { Transformer } from "@lexical/markdown";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useLexicalNodeSelection } from "@lexical/react/useLexicalNodeSelection";
import { mergeRegister } from "@lexical/utils";
import {
	$createParagraphNode,
	$createTextNode,
	$getNodeByKey,
	$getSelection,
	$isNodeSelection,
	CLICK_COMMAND,
	COMMAND_PRIORITY_HIGH,
	COMMAND_PRIORITY_LOW,
	DecoratorNode,
	type DOMConversionMap,
	type DOMConversionOutput,
	KEY_BACKSPACE_COMMAND,
	KEY_DELETE_COMMAND,
	type LexicalNode,
	type NodeKey,
	type SerializedLexicalNode,
} from "lexical";
import { useCallback, useEffect } from "react";

function PageBreakComponent({ nodeKey }: { nodeKey: NodeKey }) {
	const [editor] = useLexicalComposerContext();
	const [isSelected, setSelected, clearSelection] =
		useLexicalNodeSelection(nodeKey);

	const onDelete = useCallback(
		(event: KeyboardEvent) => {
			event.preventDefault();
			if (isSelected && $isNodeSelection($getSelection())) {
				const node = $getNodeByKey(nodeKey);
				if ($isPageBreakNode(node)) {
					node?.remove();
					return true;
				}
			}
			return false;
		},
		[isSelected, nodeKey],
	);

	useEffect(() => {
		return mergeRegister(
			editor.registerCommand(
				CLICK_COMMAND,
				(event: MouseEvent) => {
					const pbElement = editor.getElementByKey(nodeKey);
					if (event.target === pbElement) {
						if (!event.shiftKey) {
							clearSelection();
						}
						setSelected(!isSelected);
						return true;
					}
					return false;
				},
				COMMAND_PRIORITY_LOW,
			),
			editor.registerCommand(
				KEY_DELETE_COMMAND,
				onDelete,
				COMMAND_PRIORITY_LOW,
			),
			editor.registerCommand(
				KEY_BACKSPACE_COMMAND,
				onDelete,
				COMMAND_PRIORITY_LOW,
			),
		);
	}, [clearSelection, editor, isSelected, nodeKey, onDelete, setSelected]);

	useEffect(() => {
		const pbElement = editor.getElementByKey(nodeKey);

		if (pbElement !== null) {
			pbElement.className = isSelected ? "selected" : "";
		}
	}, [editor, isSelected, nodeKey]);

	return null;
}

export class PageBreakNode extends DecoratorNode<React.JSX.Element> {
	static getType(): string {
		return "page-break";
	}

	static clone(node: PageBreakNode): PageBreakNode {
		return new PageBreakNode(node.__key);
	}

	static importJSON(_serializedNode: SerializedLexicalNode): PageBreakNode {
		return $createPageBreakNode();
	}

	static importDOM(): DOMConversionMap | null {
		return {
			figure: (domNode: HTMLElement) => {
				const type = domNode.getAttribute("type");
				if (type !== PageBreakNode.getType()) return null;

				return {
					conversion: convertPageBreakElement,
					priority: COMMAND_PRIORITY_HIGH,
				};
			},
		};
	}

	exportJSON(): SerializedLexicalNode {
		return {
			type: this.getType(),
			version: 1,
		};
	}

	createDOM(): HTMLElement {
		const element = document.createElement("figure");
		element.style.pageBreakAfter = "always";
		element.setAttribute("type", this.getType());

		const hr = document.createElement("hr");
		hr.className = "editor-page-break-hr";
		element.appendChild(hr);

		return element;
	}

	getTextContent(): string {
		return "\n";
	}

	isInline(): false {
		return false;
	}

	updateDOM(): boolean {
		return false;
	}

	decorate(): React.JSX.Element {
		return <PageBreakComponent nodeKey={this.__key} />;
	}
}

function convertPageBreakElement(): DOMConversionOutput {
	return {
		node: $createPageBreakNode(),
	};
}

export function $createPageBreakNode(): PageBreakNode {
	return new PageBreakNode();
}

export function $isPageBreakNode(
	node: LexicalNode | null | undefined,
): node is PageBreakNode {
	return node instanceof PageBreakNode;
}

export const PAGE_BREAK_NODE_TRANSFORMER: Transformer = {
	export: (node) => {
		if (node.getType() === "page-break") {
			return "---\n";
		}
		return null;
	},
	regExp: /^---\s*$/,
	replace: (parentNode, _, match) => {
		const [allMatch] = match;
		const paragraphNode = $createParagraphNode();
		const textNode = $createTextNode(allMatch);
		paragraphNode.append(textNode);
		parentNode.replace($createPageBreakNode());
	},
	type: "element",
	dependencies: [],
};
