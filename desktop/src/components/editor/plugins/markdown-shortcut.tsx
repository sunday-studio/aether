import { CHECK_LIST, TRANSFORMERS } from "@lexical/markdown";
import { MarkdownShortcutPlugin as LexicalMDShortcutPlugin } from "@lexical/react/LexicalMarkdownShortcutPlugin";


export const CUSTOM_TRANSFORMERS = [
	CHECK_LIST,
	...TRANSFORMERS,
];

export const MarkdownShortcutPlugin = () => {
	return <LexicalMDShortcutPlugin transformers={CUSTOM_TRANSFORMERS} />;
};
