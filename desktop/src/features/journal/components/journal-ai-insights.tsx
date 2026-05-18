import { invoke } from '@tauri-apps/api/core';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Check, Link2, Sparkles, Tag, X } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Button } from '~/components/shared/button';
import { TextAreaField } from '~/components/shared/text-field';
import { showToast } from '~/components/shared/toast-components';
import { cn } from '~/utils/cn';
import { invalidateEntryQueries } from '../invalidate-entry-queries';

type AiSuggestionState = 'pending' | 'accepted' | 'edited' | 'dismissed';
type AiSuggestionType =
	| 'tag'
	| 'theme'
	| 'emotion'
	| 'person'
	| 'project'
	| 'open_loop'
	| 'related_entry'
	| 'related_task'
	| 'related_goal';

type JournalEntryInsight = {
	id: string;
	entryId: string;
	summary: string;
	possibleMood?: string | null;
	emotions: string[];
	energy?: string | null;
	themes: string[];
	openLoops: string[];
	state: string;
};

type JournalEntrySuggestion = {
	id: string;
	entryId: string;
	suggestionType: AiSuggestionType;
	value: string;
	editedValue?: string | null;
	targetResourceType?: string | null;
	targetResourceId?: string | null;
	state: AiSuggestionState;
};

type EntryInsightBundle = {
	insight: JournalEntryInsight;
	suggestions: JournalEntrySuggestion[];
};

type JournalAiInsightsProps = {
	entryId: string;
};

const insightQueryKey = (entryId: string) => ['journal-ai-insight', entryId] as const;

async function getEntryInsights(entryId: string) {
	return invoke<EntryInsightBundle>('get_entry_insights', {
		queryParams: { entryId },
	});
}

async function enrichJournalEntry(entryId: string) {
	return invoke<EntryInsightBundle>('enrich_journal_entry', {
		requestData: { entryId, mode: 'rules' },
	});
}

async function updateEntryInsightFields(
	insightId: string,
	fields: {
		summary: string;
		possibleMood: string;
		emotions: string[];
		energy: string;
		themes: string[];
		openLoops: string[];
	},
) {
	return invoke<EntryInsightBundle>('update_entry_insight', {
		requestData: {
			insightId,
			summary: fields.summary,
			possibleMood: fields.possibleMood,
			emotions: fields.emotions,
			energy: fields.energy,
			themes: fields.themes,
			openLoops: fields.openLoops,
		},
	});
}

async function updateSuggestion(suggestionId: string, state: AiSuggestionState) {
	return invoke('update_ai_suggestion', {
		requestData: { suggestionId, state },
	});
}

async function acceptTagSuggestion(suggestionId: string) {
	return invoke('accept_ai_tag_suggestion', {
		requestData: { suggestionId },
	});
}

async function acceptRelationSuggestion(suggestionId: string) {
	return invoke('accept_ai_relation_suggestion', {
		requestData: { suggestionId },
	});
}

export function JournalAiInsights({ entryId }: JournalAiInsightsProps) {
	const queryClient = useQueryClient();
	const [isOpen, setIsOpen] = useState(false);
	const [summaryDraft, setSummaryDraft] = useState('');
	const [possibleMoodDraft, setPossibleMoodDraft] = useState('');
	const [emotionsDraft, setEmotionsDraft] = useState('');
	const [energyDraft, setEnergyDraft] = useState('');
	const [themesDraft, setThemesDraft] = useState('');
	const [openLoopsDraft, setOpenLoopsDraft] = useState('');

	const insightQuery = useQuery({
		queryKey: insightQueryKey(entryId),
		queryFn: () => getEntryInsights(entryId),
		enabled: isOpen,
		retry: false,
	});

	useEffect(() => {
		const insight = insightQuery.data?.insight;
		if (insight) {
			setSummaryDraft(insight.summary);
			setPossibleMoodDraft(insight.possibleMood ?? '');
			setEmotionsDraft(listToText(insight.emotions));
			setEnergyDraft(insight.energy ?? '');
			setThemesDraft(listToText(insight.themes));
			setOpenLoopsDraft(listToText(insight.openLoops));
		}
	}, [insightQuery.data?.insight]);

	const refreshInsight = (bundle: EntryInsightBundle) => {
		queryClient.setQueryData(insightQueryKey(entryId), bundle);
		setSummaryDraft(bundle.insight.summary);
		setPossibleMoodDraft(bundle.insight.possibleMood ?? '');
		setEmotionsDraft(listToText(bundle.insight.emotions));
		setEnergyDraft(bundle.insight.energy ?? '');
		setThemesDraft(listToText(bundle.insight.themes));
		setOpenLoopsDraft(listToText(bundle.insight.openLoops));
	};

	const generateMutation = useMutation({
		mutationFn: () => enrichJournalEntry(entryId),
		onSuccess: bundle => {
			refreshInsight(bundle);
			showToast({ title: 'Insight draft ready' });
		},
	});

	const saveSummaryMutation = useMutation({
		mutationFn: () =>
			updateEntryInsightFields(insightQuery.data?.insight.id ?? '', {
				summary: summaryDraft,
				possibleMood: possibleMoodDraft,
				emotions: textToList(emotionsDraft),
				energy: energyDraft,
				themes: textToList(themesDraft),
				openLoops: textToList(openLoopsDraft),
			}),
		onSuccess: bundle => {
			refreshInsight(bundle);
			showToast({ title: 'Insight updated' });
		},
	});

	const suggestionMutation = useMutation({
		mutationFn: async (suggestion: JournalEntrySuggestion) => {
			if (suggestion.suggestionType === 'tag') {
				return acceptTagSuggestion(suggestion.id);
			}
			if (
				suggestion.suggestionType === 'related_entry' ||
				suggestion.suggestionType === 'related_task' ||
				suggestion.suggestionType === 'related_goal'
			) {
				return acceptRelationSuggestion(suggestion.id);
			}
			return updateSuggestion(suggestion.id, 'accepted');
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: insightQueryKey(entryId) });
			invalidateEntryQueries(queryClient);
			showToast({ title: 'Suggestion accepted' });
		},
	});

	const dismissMutation = useMutation({
		mutationFn: (suggestionId: string) => updateSuggestion(suggestionId, 'dismissed'),
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: insightQueryKey(entryId) });
		},
	});

	const pendingSuggestions =
		insightQuery.data?.suggestions.filter(suggestion => suggestion.state === 'pending') ?? [];
	const hasInsight = Boolean(insightQuery.data);

	return (
		<div className='w-full'>
			<Button
				variant='secondary'
				label={isOpen ? 'Hide AI' : 'AI'}
				tooltipContent='Toggle journal AI insights'
				iconLeft={<Sparkles className='size-3.5' />}
				onClick={() => setIsOpen(!isOpen)}
				className='h-8 px-3 py-1.5 text-xs'
			/>

			{isOpen && (
				<div className='mt-3 rounded-lg border border-(--color-border) bg-(--color-panel) p-3'>
					<div className='flex flex-wrap items-center justify-between gap-2'>
						<p className='text-sm font-medium text-(--color-primary-text)'>AI draft</p>
						<Button
							variant='ghost'
							label={hasInsight ? 'Regenerate' : 'Generate'}
							tooltipContent='Generate local insight draft'
							iconLeft={<Sparkles className='size-3.5' />}
							isDisabled={generateMutation.isPending}
							onClick={() => generateMutation.mutate()}
							className='h-8 px-3 py-1.5 text-xs'
						/>
					</div>

					{insightQuery.isError && !hasInsight ? (
						<p className='mt-3 text-sm text-(--color-secondary-text)'>No draft yet.</p>
					) : null}

					{hasInsight ? (
						<div className='mt-3 space-y-3'>
							<TextAreaField
								label='Summary'
								value={summaryDraft}
								onChange={setSummaryDraft}
							/>
							<div className='grid gap-3 md:grid-cols-2'>
								<TextAreaField
									label='Possible mood'
									value={possibleMoodDraft}
									onChange={setPossibleMoodDraft}
								/>
								<TextAreaField
									label='Energy'
									value={energyDraft}
									onChange={setEnergyDraft}
								/>
								<TextAreaField
									label='Emotions'
									value={emotionsDraft}
									onChange={setEmotionsDraft}
								/>
								<TextAreaField
									label='Themes'
									value={themesDraft}
									onChange={setThemesDraft}
								/>
							</div>
							<TextAreaField
								label='Open loops'
								value={openLoopsDraft}
								onChange={setOpenLoopsDraft}
							/>
							<div className='flex justify-end'>
								<Button
									variant='secondary'
									label='Save'
									tooltipContent='Save edited AI summary'
									iconLeft={<Check className='size-3.5' />}
									isDisabled={saveSummaryMutation.isPending}
									onClick={() => saveSummaryMutation.mutate()}
									className='h-8 px-3 py-1.5 text-xs'
								/>
							</div>

							<InsightChips insight={insightQuery.data!.insight} />

							{pendingSuggestions.length > 0 ? (
								<div className='space-y-2'>
									<p className='text-xs font-medium text-(--color-secondary-text)'>
										Suggestions
									</p>
									<div className='space-y-2'>
										{pendingSuggestions.map(suggestion => (
											<SuggestionRow
												key={suggestion.id}
												suggestion={suggestion}
												isBusy={
													suggestionMutation.isPending || dismissMutation.isPending
												}
												onAccept={() => suggestionMutation.mutate(suggestion)}
												onDismiss={() => dismissMutation.mutate(suggestion.id)}
											/>
										))}
									</div>
								</div>
							) : null}
						</div>
					) : null}
				</div>
			)}
		</div>
	);
}

function listToText(values: string[]) {
	return values.join('\n');
}

function textToList(value: string) {
	return value
		.split(/\n|,/)
		.map(item => item.trim())
		.filter(Boolean);
}

function InsightChips({ insight }: { insight: JournalEntryInsight }) {
	const chips = [
		...(insight.possibleMood ? [insight.possibleMood] : []),
		...(insight.energy ? [`energy: ${insight.energy}`] : []),
		...insight.emotions,
		...insight.themes,
	].slice(0, 8);

	if (chips.length === 0 && insight.openLoops.length === 0) return null;

	return (
		<div className='flex flex-wrap gap-1.5'>
			{chips.map(chip => (
				<span
					key={chip}
					className='rounded-full bg-neutral-100 px-2 py-1 text-xs text-neutral-700'
				>
					{chip}
				</span>
			))}
			{insight.openLoops.slice(0, 3).map(loop => (
				<span
					key={loop}
					className='rounded-full bg-amber-50 px-2 py-1 text-xs text-amber-800'
				>
					{loop}
				</span>
			))}
		</div>
	);
}

function SuggestionRow({
	suggestion,
	isBusy,
	onAccept,
	onDismiss,
}: {
	suggestion: JournalEntrySuggestion;
	isBusy: boolean;
	onAccept: () => void;
	onDismiss: () => void;
}) {
	const isRelation = suggestion.suggestionType.startsWith('related_');

	return (
		<div className='flex items-center justify-between gap-2 rounded-md border border-(--color-border) bg-(--color-background) px-2 py-2'>
			<div className='flex min-w-0 items-center gap-2'>
				{isRelation ? (
					<Link2 className='size-3.5 shrink-0 text-neutral-500' />
				) : (
					<Tag className='size-3.5 shrink-0 text-neutral-500' />
				)}
				<div className='min-w-0'>
					<p className='truncate text-sm text-(--color-primary-text)'>
						{suggestion.editedValue ?? suggestion.value}
					</p>
					<p className='text-xs text-(--color-secondary-text)'>
						{suggestion.suggestionType.replaceAll('_', ' ')}
					</p>
				</div>
			</div>
			<div className='flex shrink-0 items-center gap-1'>
				<button
					type='button'
					disabled={isBusy}
					onClick={onAccept}
					className={cn(
						'rounded-full p-1.5 text-emerald-700 hover:bg-emerald-50 disabled:opacity-50',
					)}
					aria-label='Accept suggestion'
				>
					<Check className='size-4' />
				</button>
				<button
					type='button'
					disabled={isBusy}
					onClick={onDismiss}
					className='rounded-full p-1.5 text-neutral-500 hover:bg-neutral-100 disabled:opacity-50'
					aria-label='Dismiss suggestion'
				>
					<X className='size-4' />
				</button>
			</div>
		</div>
	);
}
