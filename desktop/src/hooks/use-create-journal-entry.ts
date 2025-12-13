import { useQueryClient } from "@tanstack/react-query";
import { getGetEntriesQueryKey, useCreateEntry } from "~/aether-sdk";
import { useEditorFocusStore } from "~/store/editor-focus-state";

const placeholder =
	'{"root":{"children":[{"children":[],"direction":"ltr","format":"","indent":0,"type":"paragraph","version":1,"textFormat":0,"textStyle":""}],"direction":"ltr","format":"","indent":0,"type":"root","version":1}}';

export const useCreateJournalEntry = () => {
	const queryClient = useQueryClient();
	const { mutate } = useCreateEntry();
	const entriesQueryKey = getGetEntriesQueryKey();
	const requestFocus = useEditorFocusStore((s) => s.requestFocus);

	const createEntry = async () => {
		const now = new Date();

		mutate(
			{
				data: {
					document: placeholder,
					date: now.toISOString(),
				},
			},
			{
				onSuccess: ({ data }) => {
					console.log("data ->", data);
					queryClient.invalidateQueries({ queryKey: entriesQueryKey });
					requestFocus(data?.id ?? "");
				},
				onError: (error) => {
					console.log("error ->", error);
					console.error(error);
				},
			},
		);
	};

	return { createEntry };
};
