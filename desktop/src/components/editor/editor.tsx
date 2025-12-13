import { CodeHighlightNode, CodeNode } from "@lexical/code";
import { HashtagNode } from "@lexical/hashtag";
import { AutoLinkNode, LinkNode } from "@lexical/link";
import { ListItemNode, ListNode } from "@lexical/list";
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
import clsx from "clsx";
import type { EditorState } from "lexical";
import { useEffect, useMemo, useState } from "react";
import { useDebouncedCallback } from "use-debounce";

import { setNodePlaceholderFromSelection } from "./node-placement/utils";
import AutoLinkPlugin, { validateUrl } from "./plugins/auto-link-plugin";
import ClickableLinkPlugin from "./plugins/clickable-link-plugin";
import CodeHighlightPlugin from "./plugins/cod-highlight-plugin";
import { MarkdownShortcutPlugin } from "./plugins/markdown-shortcut";
// import SlashCommandPickerPlugin from "./plugins/slash-command-plug";
import { theme } from "./plugins/theme";
import { ReactiveFocusPlugin } from "./reactive-focus-plugin";
import { getFontFamily } from "./utils";

import "./_editor.css";

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
	return <div className={className}>Let's write something...</div>;
}

function onError(error: Error) {
	console.error(error);
}

export const EDITOR_PAGES = {
	ENTRY: "ENTRY",
	JOURNAL: "JOURNAL",
} as const;

interface EditorType {
	id: string;
	content: string | null;
	onChange: (state: string) => void;
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
	const [floatingAnchorElem, setFloatingAnchorElem] =
		useState<HTMLDivElement | null>(null);

	const onRef = (_floatingAnchorElem: HTMLDivElement) => {
		if (_floatingAnchorElem !== null) {
			setFloatingAnchorElem(_floatingAnchorElem);
		}
	};

	// biome-ignore lint/correctness/useExhaustiveDependencies: on ref will cause re-render
	const CustomContent = useMemo(
		() => (
			<div className="editor-inner" ref={onRef}>
				<ContentEditable className="editor-root" />
			</div>
		),
		[],
	);

	const editorConfig = {
		editorState: content ?? null,
		namespace: `aether-editor-${id}`,
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

			AutoLinkNode,
			LinkNode,
		],
	};

	const parseEditorOnChange = (editorState: EditorState) =>
		JSON.stringify(editorState.toJSON());

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

				{/* figure out why this is causing a performance issue */}
				{/* {floatingAnchorElem && (
					<FloatingMenuPlugin anchorElem={floatingAnchorElem} />
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
				<LinkPlugin validateUrl={validateUrl} />
				<AutoLinkPlugin />
				<MarkdownShortcutPlugin />
				<CodeHighlightPlugin />
				<ListPlugin />
				<HistoryPlugin />
				<TabIndentationPlugin />
				<HashtagPlugin />
				<ReactiveFocusPlugin id={id} />
			</div>
		</LexicalComposer>
	);
};
