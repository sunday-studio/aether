import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useEffect } from "react";
import { ResourceLinkNode } from "./resource-link-node";
import { ResourceLinkAutocomplete } from "./resource-link-autocomplete";

export function ResourceLinkPlugin() {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		if (!editor.hasNodes([ResourceLinkNode])) {
			throw new Error(
				"ResourceLinkPlugin: ResourceLinkNode is not registered on editor",
			);
		}
	}, [editor]);

	return <ResourceLinkAutocomplete />;
}
