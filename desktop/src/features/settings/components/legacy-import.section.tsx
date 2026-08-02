import { invoke } from '@tauri-apps/api/core';
import { ArchiveRestore } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';
import { Button } from '~/components/shared/button';

type LegacyImportCounts = {
	entries: number;
	tags: number;
	goals: number;
	goalInstances: number;
	tasks: number;
	subtasks: number;
	entryTags: number;
	goalTags: number;
	goalInstanceTags: number;
	taskTags: number;
};

type LegacyImportPreview = {
	sourcePath: string;
	counts: LegacyImportCounts;
};

const countLabels = [
	['entries', 'journal entries'],
	['tasks', 'tasks'],
	['goals', 'goals'],
	['goalInstances', 'goal instances'],
	['subtasks', 'subtasks'],
	['tags', 'tags'],
] as const;

export const LegacyImportSection = () => {
	const [sourcePath, setSourcePath] = useState('');
	const [preview, setPreview] = useState<LegacyImportPreview | null>(null);
	const [isPreviewing, setIsPreviewing] = useState(false);
	const [isImporting, setIsImporting] = useState(false);

	const previewImport = async () => {
		if (!sourcePath.trim()) return;
		setIsPreviewing(true);
		try {
			const result = await invoke<LegacyImportPreview>('preview_legacy_database', {
				sourcePath: sourcePath.trim(),
			});
			setPreview(result);
		} catch (error) {
			setPreview(null);
			toast.error('Legacy database could not be read', {
				description: error instanceof Error ? error.message : String(error),
			});
		} finally {
			setIsPreviewing(false);
		}
	};

	const importData = async () => {
		if (!preview || preview.sourcePath !== sourcePath.trim()) return;
		if (
			!window.confirm('Import this legacy data? Existing records with the same IDs will be kept.')
		) {
			return;
		}
		setIsImporting(true);
		try {
			const result = await invoke<LegacyImportCounts>('import_legacy_database', {
				sourcePath: sourcePath.trim(),
			});
			setPreview({ sourcePath: sourcePath.trim(), counts: result });
			toast.success('Legacy data imported', {
				description: `${result.entries} entries and ${result.tasks} tasks added; search refreshed.`,
			});
		} catch (error) {
			toast.error('Legacy data import failed', {
				description: error instanceof Error ? error.message : String(error),
			});
		} finally {
			setIsImporting(false);
		}
	};

	return (
		<div className='w-full space-y-6'>
			<div className='mb-10'>
				<h3 className='text-lg font-medium'>Legacy import</h3>
				<p className='text-sm text-(--color-secondary-text)'>
					Import data from the January 2026 self-hosted Aether backend.
				</p>
			</div>

			<div className='space-y-3'>
				<div>
					<label className='text-md' htmlFor='legacy-database-path'>
						Legacy database path
					</label>
					<p className='mt-1 text-xs text-(--color-secondary-text)'>
						Choose <code>libsql-replica/local.db</code>; keep its adjacent WAL file in place.
					</p>
				</div>
				<input
					className='w-full rounded-md border border-(--color-border) bg-(--color-panel) px-3 py-2 text-sm outline-none focus:border-(--color-button-primary-start)'
					id='legacy-database-path'
					onChange={event => {
						setSourcePath(event.target.value);
						setPreview(null);
					}}
					placeholder='/Users/you/aether-legacy-pi-backup/libsql-replica/local.db'
					spellCheck={false}
					type='text'
					value={sourcePath}
				/>
				<Button
					iconLeft={<ArchiveRestore className='size-4' strokeWidth={2.4} />}
					isDisabled={!sourcePath.trim() || isPreviewing || isImporting}
					label={isPreviewing ? 'Checking' : 'Preview import'}
					onClick={previewImport}
					tooltipContent='Read the legacy database without changing your current data'
					variant='secondary'
				/>
			</div>

			{preview && (
				<div className='space-y-4 border-t border-(--color-border) pt-4'>
					<div className='space-y-1'>
						<p className='text-sm'>Ready to import</p>
						<p className='text-xs text-(--color-secondary-text)'>
							Existing records with the same IDs are skipped; nothing is deleted or overwritten.
						</p>
					</div>
					<div className='grid grid-cols-2 gap-x-6 gap-y-2 text-xs text-(--color-secondary-text)'>
						{countLabels.map(([key, label]) => (
							<p key={key}>
								<span className='text-(--color-primary-text)'>{preview.counts[key]}</span> {label}
							</p>
						))}
					</div>
					<Button
						isDisabled={isImporting}
						label={isImporting ? 'Importing' : 'Import legacy data'}
						onClick={importData}
						tooltipContent='Add the previewed legacy data to this local Aether database'
					/>
				</div>
			)}
		</div>
	);
};
