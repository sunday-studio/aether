import type { DbEntry } from "~/aether-sdk/models";

interface EntryEditorProps {
	data: DbEntry[];
}

export const EntryEditor = ({ data }: EntryEditorProps) => {
	return <div className="bg-red-100">{data.length}</div>;
};
