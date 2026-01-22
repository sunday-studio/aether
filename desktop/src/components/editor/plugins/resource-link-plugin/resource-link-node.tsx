import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { mergeRegister } from "@lexical/utils";
import {
	$getNodeByKey,
	$getSelection,
	$isNodeSelection,
	CLICK_COMMAND,
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
import { ResourceLink } from "~/components/shared/resource-link";

function ResourceLinkComponent({
	nodeKey,
	targetType,
	targetId,
	linkText,
}: {
	nodeKey: NodeKey;
	targetType: string;
	targetId: string;
	linkText: string | null;
}) {
	const [editor] = useLexicalComposerContext();

	const onDelete = useCallback(
		(event: KeyboardEvent) => {
			event.preventDefault();
			const selection = $getSelection();
			if ($isNodeSelection(selection)) {
				const node = $getNodeByKey(nodeKey);
				if ($isResourceLinkNode(node)) {
					node?.remove();
					return true;
				}
			}
			return false;
		},
		[nodeKey],
	);

	useEffect(() => {
		return mergeRegister(
			editor.registerCommand(
				CLICK_COMMAND,
				(event: MouseEvent) => {
					const linkElement = editor.getElementByKey(nodeKey);
					if (linkElement && linkElement.contains(event.target as Node)) {
						// Don't prevent default - let the link handle navigation
						return false;
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
	}, [editor, nodeKey, onDelete]);

	return (
		<ResourceLink
			targetType={targetType}
			targetId={targetId}
			linkText={linkText}
		/>
	);
}

export type SerializedResourceLinkNode = {
	targetType: string;
	targetId: string;
	linkText: string | null;
	type: "resourceLink";
	version: 1;
};

export class ResourceLinkNode extends DecoratorNode<React.JSX.Element> {
	__targetType: string;
	__targetId: string;
	__linkText: string | null;

	static getType(): string {
		return "resourceLink";
	}

	static clone(node: ResourceLinkNode): ResourceLinkNode {
		return new ResourceLinkNode(
			node.__targetType,
			node.__targetId,
			node.__linkText,
			node.__key,
		);
	}

	constructor(
		targetType: string,
		targetId: string,
		linkText: string | null,
		key?: NodeKey,
	) {
		super(key);
		this.__targetType = targetType;
		this.__targetId = targetId;
		this.__linkText = linkText;
	}

	static importJSON(serializedNode: SerializedResourceLinkNode): ResourceLinkNode {
		const { targetType, targetId, linkText } = serializedNode;
		return $createResourceLinkNode(targetType, targetId, linkText);
	}

	static importDOM(): DOMConversionMap | null {
		return {
			span: (domNode: HTMLElement) => {
				const type = domNode.getAttribute("data-resource-link");
				if (type !== "true") return null;

				const targetType = domNode.getAttribute("data-target-type");
				const targetId = domNode.getAttribute("data-target-id");
				const linkText = domNode.getAttribute("data-link-text");

				if (!targetType || !targetId) return null;

				return {
					conversion: () => ({
						node: $createResourceLinkNode(
							targetType,
							targetId,
							linkText || null,
						),
					}),
					priority: 1,
				};
			},
		};
	}

	exportJSON(): SerializedResourceLinkNode {
		return {
			targetType: this.__targetType,
			targetId: this.__targetId,
			linkText: this.__linkText,
			type: "resourceLink",
			version: 1,
		};
	}

	createDOM(): HTMLElement {
		const element = document.createElement("span");
		element.setAttribute("data-resource-link", "true");
		element.setAttribute("data-target-type", this.__targetType);
		element.setAttribute("data-target-id", this.__targetId);
		if (this.__linkText) {
			element.setAttribute("data-link-text", this.__linkText);
		}
		return element;
	}

	getTextContent(): string {
		return this.__linkText || `[[${this.__targetType}:${this.__targetId}]]`;
	}

	isInline(): true {
		return true;
	}

	updateDOM(): boolean {
		return false;
	}

	decorate(): React.JSX.Element {
		return (
			<ResourceLinkComponent
				nodeKey={this.__key}
				targetType={this.__targetType}
				targetId={this.__targetId}
				linkText={this.__linkText}
			/>
		);
	}

	setTargetType(targetType: string): void {
		const writable = this.getWritable();
		writable.__targetType = targetType;
	}

	setTargetId(targetId: string): void {
		const writable = this.getWritable();
		writable.__targetId = targetId;
	}

	setLinkText(linkText: string | null): void {
		const writable = this.getWritable();
		writable.__linkText = linkText;
	}
}

export function $createResourceLinkNode(
	targetType: string,
	targetId: string,
	linkText: string | null = null,
): ResourceLinkNode {
	return new ResourceLinkNode(targetType, targetId, linkText);
}

export function $isResourceLinkNode(
	node: LexicalNode | null | undefined,
): node is ResourceLinkNode {
	return node instanceof ResourceLinkNode;
}
