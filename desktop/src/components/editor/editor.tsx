import { useEffect, useMemo, useState } from "react";

import { CodeHighlightNode, CodeNode } from "@lexical/code";
import { HashtagNode } from "@lexical/hashtag";
import { AutoLinkNode, LinkNode } from "@lexical/link";
import { ListItemNode, ListNode } from "@lexical/list";
import { CheckListPlugin } from "@lexical/react/LexicalCheckListPlugin";
import { LexicalComposer } from "@lexical/react/LexicalComposer";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import { HashtagPlugin } from "@lexical/react/LexicalHashtagPlugin";
import { HistoryPlugin } from "@lexical/react/LexicalHistoryPlugin";
import { LinkPlugin } from "@lexical/react/LexicalLinkPlugin";
import { ListPlugin } from "@lexical/react/LexicalListPlugin";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { TabIndentationPlugin } from "@lexical/react/LexicalTabIndentationPlugin";
import { HeadingNode, QuoteNode } from "@lexical/rich-text";
import { TableCellNode, TableNode, TableRowNode } from "@lexical/table";
import clsx from "clsx";
import { EditorState } from "lexical";
import { useDebouncedCallback } from "use-debounce";

import "./_editor.css";

// import CodeActionPlugin from "./code-action-plugin";
import { FloatingMenuPlugin } from "./floating-menu-plugin/floating-menu-plugin";
import { setNodePlaceholderFromSelection } from "./node-placement/utils";
import "./_editor.css";
import ClickableLinkPlugin from "./plugins/clickable-link-plugin";
// import AutoLinkPlugin, { validateUrl } from "./plugins/auto-link-plugin";
// import CodeHighlightPlugin from "./plugins/cod-highlight-plugin";
// import { MarkdownShortcutPlugin } from "./plugins/markdown-shortcut";
// import PageBreakPlugin from "./plugins/page-break-plugin/page-break-plugin";
// import SlashCommandPickerPlugin from "./plugins/slash-command-plug";
// import TabFocusPlugin from "./plugins/tab-focus-plugin";
import DraggableBlockPlugin from "./plugins/draggable-block";
import { PageBreakNode } from "./plugins/page-break-plugin/nodes/page-break-node";
import { theme } from "./plugins/theme";
import { getFontFamily } from "./utils";

const ONCHANGE_DEBOUNCE_TIME = 750;
const ONHISTORYCHANGE_DEBOUNCE_TIME = 600000; // 10 minutes in milliseconds

const OnChangePlugin = ({
	onChange,
}: {
	onChange: (editorState: EditorState) => void;
}) => {
	const [editor] = useLexicalComposerContext();
	useEffect(() => {
		return editor.registerUpdateListener(({ editorState }) => {
			setNodePlaceholderFromSelection(editor);
			onChange(editorState);
		});
	}, [editor, onChange]);
	return null;
};

function Placeholder({ className }: { className: string }) {
	return (
		<div className={className}>I'm a placeholder, let's write something...</div>
	);
}

function onError(error: any) {
	console.error(error);
}

export const EDITOR_PAGES = {
	ENTRY: "ENTRY",
	JOURNAL: "JOURNAL",
} as const;

interface EditorType {
	id: string;
	content: string | null;
	onChange: (state: any) => void;
	placeholderClassName?: string;
	onHistoryChange: (state: string) => void;
}

export const Editor = ({
	id,
	content,
	onChange,
	placeholderClassName = "editor-placeholder",
	onHistoryChange,
}: EditorType) => {
	const CustomContent = useMemo(() => {
		return (
			<div className="editor-inner">
				<ContentEditable className="editor-root" />
			</div>
		);
	}, []);

	const editorConfig = {
		editorState: content ?? null,
		namespace: "ContentEditor",
		theme,
		onError,
		nodes: [
			HashtagNode,
			HeadingNode,
			ListNode,
			ListItemNode,
			QuoteNode,
			CodeNode,
			CodeHighlightNode,
			TableNode,
			TableCellNode,
			TableRowNode,
			AutoLinkNode,
			LinkNode,
			PageBreakNode,
		],
	};

	const parseEditorOnChange = (editorState: EditorState) => {
		return JSON.stringify(editorState.toJSON());
	};

	const debouncedOnChange = useDebouncedCallback(async (state: string) => {
		onChange(state);
	}, ONCHANGE_DEBOUNCE_TIME);

	const debouncedOnHistoryChange = useDebouncedCallback(
		async (state: string) => {
			onHistoryChange(state);
		},
		ONHISTORYCHANGE_DEBOUNCE_TIME,
	);

	const fontFamilyClass = getFontFamily();

	return (
		<LexicalComposer initialConfig={editorConfig} key={id}>
			<div className={clsx("editor-wrapper", fontFamilyClass)}>
				<RichTextPlugin
					contentEditable={CustomContent}
					placeholder={<Placeholder className={placeholderClassName} />}
					ErrorBoundary={LexicalErrorBoundary}
				/>
				{/* {floatingAnchorElem && (
					<>
						<DraggableBlockPlugin anchorElem={floatingAnchorElem} />
						<FloatingMenuPlugin anchorElem={floatingAnchorElem} />
					</>
				)} */}
				<ClickableLinkPlugin />
				<OnChangePlugin
					onChange={(state) => {
						const editorStateJSON = parseEditorOnChange(state);
						debouncedOnChange(editorStateJSON);
						debouncedOnHistoryChange(editorStateJSON);
					}}
				/>
				{/* <SlashCommandPickerPlugin /> */}
				{/* <TabFocusPlugin /> */}
				{/* <LinkPlugin validateUrl={validateUrl} /> */}
				{/* <AutoLinkPlugin /> */}
				{/* <MarkdownShortcutPlugin />
				<CodeHighlightPlugin />
				<PageBreakPlugin /> */}
				<ListPlugin />
				<CheckListPlugin />
				<HistoryPlugin />
				<TabIndentationPlugin />
				<HashtagPlugin />
			</div>
		</LexicalComposer>
	);
};
