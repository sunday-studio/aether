#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const defaultDbPath = path.resolve(scriptDir, '../src-tauri/target/libsql-replica-dev/local.db');
const namespace = 'seed-v1';

const args = new Map();
for (let index = 2; index < process.argv.length; index += 1) {
	const arg = process.argv[index];
	if (!arg.startsWith('--')) continue;
	const [key, inlineValue] = arg.slice(2).split('=');
	const nextValue = process.argv[index + 1];
	if (inlineValue != null) {
		args.set(key, inlineValue);
	} else if (nextValue && !nextValue.startsWith('--')) {
		args.set(key, nextValue);
		index += 1;
	} else {
		args.set(key, true);
	}
}

const dbPath = path.resolve(String(args.get('db') ?? defaultDbPath));
const days = Number(args.get('days') ?? 84);
const printSql = args.has('print-sql');

if (!printSql && !existsSync(dbPath)) {
	console.error(`Could not find database at ${dbPath}`);
	console.error('Run `pnpm tauri dev` once from desktop/ so migrations create the local dev DB.');
	process.exit(1);
}

const now = new Date();
const todayUtc = startOfUtcDay(now);
const startDate = addDays(todayUtc, -(days - 1));
const sql = buildSeedSql();

if (printSql) {
	process.stdout.write(sql);
	process.exit(0);
}

const tempDir = mkdtempSync(path.join(tmpdir(), 'aether-seed-'));
const sqlPath = path.join(tempDir, 'seed-v1-demo-data.sql');
writeFileSync(sqlPath, sql);

const result = spawnSync('sqlite3', ['-bail', dbPath, `.read ${sqlPath}`], {
	stdio: 'inherit',
});

rmSync(tempDir, { recursive: true, force: true });

if (result.error) {
	console.error(result.error.message);
	process.exit(1);
}

if (result.status !== 0) {
	process.exit(result.status ?? 1);
}

console.log(`Seeded Aether v1 demo data into ${dbPath}`);
console.log(`Window: ${iso(startDate)} through ${iso(addDays(todayUtc, 14))}`);
console.log('Created: 20 tags, 84 journal entries, 6 goals, 110+ goal instances, 180+ tasks, 500+ subtasks, bookmarks, canvases, media, transcriptions, links, activities, and search documents.');

function buildSeedSql() {
	const statements = [
		'.bail on',
		'PRAGMA foreign_keys = ON;',
		'BEGIN TRANSACTION;',
		"INSERT OR IGNORE INTO _sync_meta (key, value) VALUES ('_suppress_triggers', '0');",
		"UPDATE _sync_meta SET value = '1' WHERE key = '_suppress_triggers';",
		...deleteSeedStatements(),
		...settingsStatements(),
		...dataStatements(),
		"UPDATE _sync_meta SET value = '0' WHERE key = '_suppress_triggers';",
		'COMMIT;',
	];

	return `${statements.join('\n')}\n`;
}

function deleteSeedStatements() {
	return [
		`DELETE FROM search_embeddings WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM search_documents WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM resource_links WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM bookmark_tags WHERE bookmark_id LIKE '${namespace}-%' OR tag_id LIKE '${namespace}-%';`,
		`DELETE FROM bookmarks WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM audio_transcriptions WHERE id LIKE '${namespace}-%' OR media_id LIKE '${namespace}-%';`,
		`DELETE FROM media_items WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM canvases WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM task_tags WHERE task_id LIKE '${namespace}-%' OR tag_id LIKE '${namespace}-%';`,
		`DELETE FROM subtasks WHERE id LIKE '${namespace}-%' OR task_id LIKE '${namespace}-%';`,
		`DELETE FROM tasks WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM goal_instance_tags WHERE goal_instance_id LIKE '${namespace}-%' OR tag_id LIKE '${namespace}-%';`,
		`DELETE FROM goal_instances WHERE id LIKE '${namespace}-%' OR goal_id LIKE '${namespace}-%';`,
		`DELETE FROM goal_tags WHERE goal_id LIKE '${namespace}-%' OR tag_id LIKE '${namespace}-%';`,
		`DELETE FROM goals WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM entry_tags WHERE entry_id LIKE '${namespace}-%' OR tag_id LIKE '${namespace}-%';`,
		`DELETE FROM entries WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM activities WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM tags WHERE id LIKE '${namespace}-%';`,
		`DELETE FROM _sync_outbox WHERE entity_id LIKE '${namespace}-%';`,
	];
}

function settingsStatements() {
	return [
		upsert('settings', {
			key: 'app.onboarding_completed',
			value: 'true',
			updated_at: iso(now),
		}),
		upsert('settings', {
			key: 'user.display_name',
			value: 'Aether Reviewer',
			updated_at: iso(now),
		}),
		upsert('settings', {
			key: 'sync.media_sync_policy',
			value: 'on_demand',
			updated_at: iso(now),
		}),
		upsert('settings', {
			key: 'transcription.default_provider',
			value: 'openai',
			updated_at: iso(now),
		}),
	];
}

function dataStatements() {
	const rows = [];
	const tags = buildTags();
	const entries = buildEntries(tags);
	const goals = buildGoals(tags);
	const goalInstances = buildGoalInstances(goals);
	const tasks = buildTasks(tags, goals, goalInstances, entries);
	const subtasks = buildSubtasks(tasks);
	const bookmarks = buildBookmarks(tags);
	const canvases = buildCanvases(tags, goals, tasks, entries);
	const mediaItems = buildMediaItems(entries, tasks);
	const transcriptions = buildTranscriptions(mediaItems);
	const links = buildResourceLinks(entries, tasks, goals, bookmarks, canvases);
	const activities = buildActivities(entries, tasks, goals, bookmarks, canvases);
	const searchDocuments = buildSearchDocuments(tags, entries, goals, tasks, bookmarks);

	for (const tag of tags) rows.push(insertSyncRow('tags', tag));
	for (const entry of entries) rows.push(insertSyncRow('entries', without(entry, ['tag_ids', 'plain_text', 'title'])));
	for (const entry of entries) {
		for (const tagId of entry.tag_ids) {
			rows.push(insertJoin('entry_tags', { entry_id: entry.id, tag_id: tagId }, `${entry.id}|${tagId}`, entry.created_at));
		}
	}
	for (const goal of goals) rows.push(insertSyncRow('goals', without(goal, ['tag_ids'])));
	for (const goal of goals) {
		for (const tagId of goal.tag_ids) {
			rows.push(insertJoin('goal_tags', { goal_id: goal.id, tag_id: tagId }, `${goal.id}|${tagId}`, goal.created_at));
		}
	}
	for (const instance of goalInstances) rows.push(insertSyncRow('goal_instances', without(instance, ['tag_ids'])));
	for (const instance of goalInstances) {
		for (const tagId of instance.tag_ids) {
			rows.push(insertJoin('goal_instance_tags', { goal_instance_id: instance.id, tag_id: tagId }, `${instance.id}|${tagId}`, instance.created_at));
		}
	}
	for (const task of tasks) rows.push(insertSyncRow('tasks', without(task, ['tag_ids'])));
	for (const task of tasks) {
		for (const tagId of task.tag_ids) {
			rows.push(insertJoin('task_tags', { task_id: task.id, tag_id: tagId }, `${task.id}|${tagId}`, task.created_at));
		}
	}
	for (const subtask of subtasks) rows.push(insertSyncRow('subtasks', subtask));
	for (const bookmark of bookmarks) rows.push(insertSyncRow('bookmarks', without(bookmark, ['tag_ids'])));
	for (const bookmark of bookmarks) {
		for (const tagId of bookmark.tag_ids) {
			rows.push(insertJoin('bookmark_tags', { bookmark_id: bookmark.id, tag_id: tagId }, `${bookmark.id}|${tagId}`, bookmark.created_at));
		}
	}
	for (const canvas of canvases) rows.push(insertSyncRow('canvases', canvas));
	for (const media of mediaItems) rows.push(insertSyncRow('media_items', media));
	for (const transcription of transcriptions) rows.push(insertSyncRow('audio_transcriptions', transcription));
	for (const link of links) rows.push(insertSyncRow('resource_links', link));
	for (const activityRow of activities) rows.push(insertSyncRow('activities', activityRow));
	for (const doc of searchDocuments) rows.push(upsert('search_documents', doc));

	return rows;
}

function buildTags() {
	const names = [
		'daily-review',
		'deep-work',
		'launch',
		'design',
		'engineering',
		'research',
		'health',
		'writing',
		'finance',
		'personal',
		'travel',
		'reading',
		'meeting',
		'ideas',
		'blocked',
		'waiting',
		'urgent',
		'someday',
		'ai',
		'sync',
	];

	return names.map((name, index) => {
		const createdAt = addHours(startDate, 7 + index);
		return withSync({
			id: `${namespace}-tag-${name}`,
			name,
			created_at: iso(createdAt),
			updated_at: iso(createdAt),
			deleted_at: null,
		}, createdAt);
	});
}

function buildEntries(tags) {
	const tag = tagMap(tags);
	const themes = [
		['deep-work', 'engineering'],
		['design', 'ideas'],
		['daily-review', 'personal'],
		['research', 'ai'],
		['health', 'personal'],
		['launch', 'urgent'],
		['writing', 'reading'],
		['sync', 'engineering'],
		['finance', 'personal'],
		['meeting', 'waiting'],
		['travel', 'ideas'],
		['blocked', 'launch'],
	];
	const prompts = [
		'Morning planning',
		'Focus block notes',
		'Decision log',
		'Reflection',
		'Review and reset',
		'Prototype notes',
		'Customer insight',
		'Sync check',
		'Release prep',
		'Open questions',
		'Reading note',
		'Evening wrap',
	];

	return Array.from({ length: days }, (_, index) => {
		const day = addDays(startDate, index);
		const createdAt = addHours(day, 8 + (index % 10));
		const title = `${prompts[index % prompts.length]} ${formatShort(day)}`;
		const mood = ['steady', 'curious', 'a little noisy', 'clear', 'usefully restless'][index % 5];
		const plainText = `${title}. Energy felt ${mood}. Main thread was ${themes[index % themes.length].join(' and ')}. Captured one concrete next action and one loose idea to revisit.`;
		const tagIds = themes[index % themes.length].map(name => tag[name]);

		return withSync({
			id: `${namespace}-entry-${dateId(day)}`,
			document: lexicalDocument(title, [
				plainText,
				`What mattered: ${['shipping the small thing', 'reducing ambiguity', 'protecting attention', 'following up clearly'][index % 4]}.`,
				`Next: ${['draft the release note', 'check the task board', 'review tags', 'turn this into a goal task'][index % 4]}.`,
			]),
			created_at: iso(createdAt),
			is_pinned: index % 21 === 0 ? 1 : 0,
			is_archived: index < 12 && index % 6 === 0 ? 1 : 0,
			is_deleted: 0,
			updated_at: iso(addMinutes(createdAt, 18)),
			deleted_at: null,
			tag_ids: tagIds,
			title,
			plain_text: plainText,
		}, createdAt);
	});
}

function buildGoals(tags) {
	const tag = tagMap(tags);
	const specs = [
		['daily-writing', 'Write a useful daily note', 'Capture one honest daily review and one next action.', 0, 'daily', 1, ['writing', 'daily-review']],
		['weekly-review', 'Complete weekly review', 'Close loops, prune stale tasks, and pick the next focus.', 0, 'weekly', 1, ['daily-review', 'personal']],
		['v1-launch', 'Prepare Aether v1 release', 'Review onboarding, sync, search, journal, tasks, and packaging.', 1, null, null, ['launch', 'engineering', 'urgent']],
		['strength-routine', 'Keep strength routine', 'Three simple movement sessions a week with notes.', 0, 'weekly', 1, ['health', 'personal']],
		['monthly-finance', 'Monthly finance close', 'Review subscriptions, receipts, invoices, and savings.', 0, 'monthly', 1, ['finance', 'personal']],
		['reading-sprint', 'Read and synthesize', 'Read in two-week sprints and turn notes into action.', 0, 'bi-weekly', 2, ['reading', 'research']],
	];

	return specs.map(([slug, name, description, isNonRecurring, recurrenceType, interval, tagNames], index) => {
		const createdAt = addDays(startDate, index * 3);
		return withSync({
			id: `${namespace}-goal-${slug}`,
			name,
			description,
			is_non_recurring: isNonRecurring,
			recurrence_type: recurrenceType,
			recurrence_interval: interval,
			recurrence_anchor: recurrenceType ? iso(startDate) : null,
			recurrence_meta: recurrenceType === 'weekly' ? json({ weekStartsOn: 1 }) : recurrenceType === 'monthly' ? json({ dayOfMonth: 1 }) : null,
			timezone: 'Europe/Amsterdam',
			created_at: iso(createdAt),
			updated_at: iso(addHours(createdAt, 1)),
			deleted_at: null,
			tag_ids: tagNames.map(tagName => tag[tagName]),
		}, createdAt);
	});
}

function buildGoalInstances(goals) {
	const instances = [];
	for (const goal of goals) {
		const cadence = goal.recurrence_type;
		const dates = [];
		if (goal.is_non_recurring) {
			dates.push(addDays(startDate, 10));
		} else if (cadence === 'daily') {
			for (let index = 0; index < days; index += 1) dates.push(addDays(startDate, index));
		} else if (cadence === 'weekly') {
			for (let index = 0; index < days; index += 7) dates.push(addDays(startDate, index));
		} else if (cadence === 'bi-weekly') {
			for (let index = 0; index < days; index += 14) dates.push(addDays(startDate, index));
		} else if (cadence === 'monthly') {
			for (let index = 0; index < days; index += 28) dates.push(addDays(startDate, index));
		}

		for (const [index, periodStart] of dates.entries()) {
			const periodEnd = goal.is_non_recurring ? null : iso(addDays(periodStart, cadence === 'daily' ? 1 : cadence === 'monthly' ? 28 : 7 * Number(goal.recurrence_interval ?? 1)));
			const status = periodStart < addDays(todayUtc, -2) ? (index % 9 === 0 ? 'skipped' : 'completed') : 'active';
			instances.push(withSync({
				id: `${goal.id}-instance-${dateId(periodStart)}`,
				goal_id: goal.id,
				period_start: iso(periodStart),
				period_end: periodEnd,
				status,
				created_at: iso(addHours(periodStart, 6)),
				updated_at: iso(addHours(periodStart, 6)),
				deleted_at: null,
				tag_ids: goal.tag_ids.slice(0, 2),
			}, periodStart));
		}
	}
	return instances;
}

function buildTasks(tags, goals, goalInstances, entries) {
	const tag = tagMap(tags);
	const tasks = [];
	let counter = 0;
	for (const instance of goalInstances) {
		const goal = goals.find(item => item.id === instance.goal_id);
		const createdAt = addHours(new Date(instance.period_start), 9);
		const dueAt = addHours(new Date(instance.period_start), goal?.recurrence_type === 'daily' ? 20 : 17);
		const isCompleted = dueAt < addDays(todayUtc, -1) && counter % 8 !== 0 ? 1 : 0;
		const extraTag = counter % 5 === 0 ? tag.blocked : counter % 4 === 0 ? tag.waiting : tag['deep-work'];
		tasks.push(withSync({
			id: `${namespace}-task-goal-${counter.toString().padStart(3, '0')}`,
			title: `${goal?.name ?? 'Goal'} checkpoint`,
			description: `Seeded goal task for ${formatShort(new Date(instance.period_start))}. Review this in the goal view, task inbox, and search.`,
			is_completed: isCompleted,
			due_date: iso(dueAt),
			goal_instance_id: instance.id,
			goal_id: instance.goal_id,
			created_at: iso(createdAt),
			updated_at: iso(addMinutes(createdAt, 35)),
			deleted_at: null,
			tag_ids: unique([...(goal?.tag_ids ?? []), extraTag]),
		}, createdAt));
		counter += 1;
	}

	const standaloneTitles = [
		'Review onboarding copy in desktop shell',
		'Check journal timeline density',
		'Verify tag selector with many tags',
		'Record sync setup questions',
		'Triage release-blocking bugs',
		'Write launch checklist notes',
		'Compare search results for seeded content',
		'Prepare demo workspace reset plan',
	];

	for (let index = 0; index < days; index += 2) {
		const day = addDays(startDate, index);
		const createdAt = addHours(day, 10 + (index % 6));
		const dueAt = addDays(createdAt, (index % 9) - 2);
		const linkedEntry = entries[index % entries.length];
		tasks.push(withSync({
			id: `${namespace}-task-standalone-${dateId(day)}`,
			title: standaloneTitles[index % standaloneTitles.length],
			description: `Standalone seeded task tied to ${linkedEntry.title}. It has subtasks, tags, due dates, and mixed completion state.`,
			is_completed: dueAt < todayUtc && index % 4 !== 0 ? 1 : 0,
			due_date: iso(dueAt),
			goal_instance_id: null,
			goal_id: null,
			created_at: iso(createdAt),
			updated_at: iso(addMinutes(createdAt, 24)),
			deleted_at: null,
			tag_ids: unique([tag.launch, tag.engineering, index % 3 === 0 ? tag.urgent : tag.someday]),
		}, createdAt));
	}

	return tasks;
}

function buildSubtasks(tasks) {
	const labels = ['Draft', 'Review', 'Follow up', 'Close loop'];
	const subtasks = [];
	for (const [taskIndex, task] of tasks.entries()) {
		const count = 2 + (taskIndex % 3);
		for (let index = 0; index < count; index += 1) {
			const createdAt = addMinutes(new Date(task.created_at), index + 1);
			subtasks.push(withSync({
				id: `${task.id}-subtask-${index + 1}`,
				title: `${labels[index]}: ${task.title.toLowerCase()}`,
				is_completed: task.is_completed || index === 0 && taskIndex % 4 !== 0 ? 1 : 0,
				task_id: task.id,
				order_index: index,
				created_at: iso(createdAt),
				updated_at: iso(addMinutes(createdAt, 12)),
				deleted_at: null,
			}, createdAt));
		}
	}
	return subtasks;
}

function buildBookmarks(tags) {
	const tag = tagMap(tags);
	const specs = [
		['https://tauri.app/', 'Tauri Documentation', 'Desktop shell reference for app packaging and APIs.', 'Tauri', 'documentation', ['engineering']],
		['https://lexical.dev/', 'Lexical Editor', 'Rich text editor architecture used by the journal surface.', 'Meta Open Source', 'documentation', ['engineering', 'writing']],
		['https://jsoncanvas.org/', 'JSON Canvas Spec', 'Portable canvas format used by Aether canvas boards.', 'JSON Canvas', 'article', ['design', 'research']],
		['https://github.com/sunday-studio/aether', 'Aether Repository', 'Project repository and release work.', 'GitHub', 'repo', ['launch', 'engineering']],
		['https://www.sqlite.org/fts5.html', 'SQLite FTS5', 'Full-text search reference for seeded search review.', 'SQLite', 'documentation', ['research']],
		['https://www.openai.com/', 'OpenAI', 'Provider option for AI transcription and summaries.', 'OpenAI', 'provider', ['ai']],
		['https://console.groq.com/', 'Groq Console', 'Fast hosted AI provider option.', 'Groq', 'provider', ['ai']],
		['https://www.docker.com/', 'Docker', 'Self-hosted sync server deployment dependency.', 'Docker', 'tool', ['sync']],
		['https://mobbin.com/', 'Mobbin', 'Design inspiration source for onboarding patterns.', 'Mobbin', 'design', ['design']],
		['https://www.relay.fm/cortex', 'Cortex Notes', 'Personal systems and review inspiration.', 'Relay', 'podcast', ['personal', 'ideas']],
		['https://www.nngroup.com/articles/onboarding-tutorials/', 'Onboarding UX Notes', 'Reference for first-run flow review.', 'NN/g', 'article', ['design', 'research']],
		['https://github.com/protocolbuffers/protobuf', 'Serialization Notes', 'Reference for thinking about sync payload evolution.', 'GitHub', 'repo', ['sync', 'engineering']],
	];
	return specs.map(([url, title, description, siteName, contentType, tagNames], index) => {
		const createdAt = addDays(startDate, index * 5);
		return withSync({
			id: `${namespace}-bookmark-${String(index + 1).padStart(2, '0')}`,
			url,
			title,
			description,
			image_url: null,
			favicon_url: `${new URL(url).origin}/favicon.ico`,
			site_name: siteName,
			author: siteName,
			published_at: iso(addDays(createdAt, -20)),
			content_type: contentType,
			metadata_json: json({ seeded: true, reviewSurface: 'bookmarks' }),
			is_archived: index % 5 === 0 ? 1 : 0,
			is_deleted: 0,
			created_at: iso(createdAt),
			updated_at: iso(addHours(createdAt, 2)),
			deleted_at: null,
			tag_ids: tagNames.map(name => tag[name]),
		}, createdAt);
	});
}

function buildCanvases(tags, goals, tasks, entries) {
	const boards = [
		['release-map', 'V1 release map', ['launch', 'engineering']],
		['research-web', 'Research web', ['research', 'ideas']],
		['personal-system', 'Personal operating system', ['personal', 'health']],
	];
	const tag = tagMap(tags);
	return boards.map(([slug, boardName, tagNames], index) => {
		const createdAt = addDays(startDate, 14 + index * 18);
		const canvasData = {
			nodes: [
				{ id: `${slug}-entry`, type: 'text', x: 40, y: 60, width: 260, height: 120, text: `Entry: ${entries[index * 9].title}` },
				{ id: `${slug}-goal`, type: 'text', x: 380, y: 80, width: 260, height: 120, text: `Goal: ${goals[index].name}` },
				{ id: `${slug}-task`, type: 'text', x: 210, y: 260, width: 280, height: 120, text: `Task: ${tasks[index * 12].title}` },
			],
			edges: [
				{ id: `${slug}-edge-1`, fromNode: `${slug}-entry`, toNode: `${slug}-goal` },
				{ id: `${slug}-edge-2`, fromNode: `${slug}-goal`, toNode: `${slug}-task` },
			],
		};
		return withSync({
			id: `${namespace}-canvas-${slug}`,
			name: boardName,
			canvas_data: json({ ...canvasData, tags: tagNames.map(tagName => tag[tagName]) }),
			created_at: iso(createdAt),
			updated_at: iso(addHours(createdAt, 3)),
			deleted_at: null,
		}, createdAt);
	});
}

function buildMediaItems(entries, tasks) {
	const items = [];
	for (let index = 0; index < 8; index += 1) {
		const entry = entries[index * 7];
		const createdAt = addMinutes(new Date(entry.created_at), 15);
		items.push(withSync({
			id: `${namespace}-media-entry-audio-${index + 1}`,
			entity_type: 'entry',
			entity_id: entry.id,
			media_type: 'audio',
			file_path: `seed/audio/journal-note-${index + 1}.m4a`,
			metadata: json({ durationSeconds: 120 + index * 34, seeded: true }),
			created_at: iso(createdAt),
			updated_at: iso(createdAt),
		}, createdAt));
	}
	for (let index = 0; index < 4; index += 1) {
		const task = tasks[index * 13];
		const createdAt = addMinutes(new Date(task.created_at), 20);
		items.push(withSync({
			id: `${namespace}-media-task-image-${index + 1}`,
			entity_type: 'task',
			entity_id: task.id,
			media_type: 'image',
			file_path: `seed/images/task-reference-${index + 1}.png`,
			metadata: json({ width: 1440, height: 900, seeded: true }),
			created_at: iso(createdAt),
			updated_at: iso(createdAt),
		}, createdAt));
	}
	return items;
}

function buildTranscriptions(mediaItems) {
	return mediaItems
		.filter(item => item.media_type === 'audio')
		.map((media, index) => {
			const createdAt = addMinutes(new Date(media.created_at), 3);
			return withSync({
				id: `${namespace}-transcription-${index + 1}`,
				media_id: media.id,
				transcription_text: `Seeded transcription ${index + 1}: talked through the review, named one blocker, and chose the next concrete action.`,
				provider: index % 2 === 0 ? 'openai' : 'groq',
				provider_config: json({ model: index % 2 === 0 ? 'whisper-1' : 'whisper-large-v3' }),
				confidence_score: 0.88 + index * 0.01,
				status: index === 7 ? 'failed' : 'complete',
				error_message: index === 7 ? 'Seeded failure for error-state review.' : null,
				is_active: index === 7 ? 0 : 1,
				created_at: iso(createdAt),
			}, createdAt);
		});
}

function buildResourceLinks(entries, tasks, goals, bookmarks, canvases) {
	const links = [];
	for (let index = 0; index < 30; index += 1) {
		const createdAt = addMinutes(new Date(entries[index * 2].created_at), 30);
		links.push(withSync({
			id: `${namespace}-link-entry-task-${index + 1}`,
			source_type: 'entry',
			source_id: entries[index * 2].id,
			target_type: 'task',
			target_id: tasks[index * 3].id,
			link_text: 'next action',
			created_at: iso(createdAt),
		}, createdAt));
	}
	for (let index = 0; index < goals.length; index += 1) {
		const createdAt = addDays(startDate, 20 + index);
		links.push(withSync({
			id: `${namespace}-link-goal-bookmark-${index + 1}`,
			source_type: 'goal',
			source_id: goals[index].id,
			target_type: 'bookmark',
			target_id: bookmarks[index].id,
			link_text: 'reference',
			created_at: iso(createdAt),
		}, createdAt));
	}
	for (let index = 0; index < canvases.length; index += 1) {
		const createdAt = addDays(startDate, 30 + index);
		links.push(withSync({
			id: `${namespace}-link-canvas-goal-${index + 1}`,
			source_type: 'canvas',
			source_id: canvases[index].id,
			target_type: 'goal',
			target_id: goals[index].id,
			link_text: 'map',
			created_at: iso(createdAt),
		}, createdAt));
	}
	return links;
}

function buildActivities(entries, tasks, goals, bookmarks, canvases) {
	const resources = [
		...entries.slice(0, 45).map(item => ['entry', item.id, item.created_at]),
		...tasks.slice(0, 90).map(item => ['task', item.id, item.created_at]),
		...goals.map(item => ['goal', item.id, item.created_at]),
		...bookmarks.map(item => ['bookmark', item.id, item.created_at]),
		...canvases.map(item => ['canvas', item.id, item.created_at]),
	];
	return resources.flatMap(([entityType, entityId, createdAt], index) => {
		const base = new Date(createdAt);
		const rows = [activity(index, 'create', entityType, entityId, base)];
		if (index % 3 === 0) rows.push(activity(`${index}-u`, 'update', entityType, entityId, addHours(base, 2)));
		if (entityType === 'task' && index % 4 === 0) rows.push(activity(`${index}-c`, 'complete', entityType, entityId, addHours(base, 5)));
		return rows;
	});
}

function activity(id, actionType, entityType, entityId, createdAt) {
	return withSync({
		id: `${namespace}-activity-${id}`,
		action_type: actionType,
		entity_type: entityType,
		entity_id: entityId,
		created_at: iso(createdAt),
		metadata: json({ seeded: true, source: 'seed-v1-demo-data' }),
	}, createdAt);
}

function buildSearchDocuments(tags, entries, goals, tasks, bookmarks) {
	const documents = [];
	for (const entry of entries) {
		documents.push(searchDocument('entry', entry.id, entry.title, entry.plain_text, entry.updated_at));
	}
	for (const goal of goals) {
		documents.push(searchDocument('goal', goal.id, goal.name, goal.description ?? '', goal.updated_at));
	}
	for (const task of tasks) {
		documents.push(searchDocument('task', task.id, task.title, task.description ?? '', task.updated_at));
	}
	for (const tag of tags) {
		documents.push(searchDocument('tag', tag.id, tag.name, `Tag ${tag.name}`, tag.updated_at));
	}
	for (const bookmark of bookmarks) {
		documents.push(searchDocument('bookmark', bookmark.id, bookmark.title, bookmark.description ?? '', bookmark.updated_at));
	}
	return documents;
}

function searchDocument(resourceType, resourceId, title, text, sourceUpdatedAt) {
	const source = `${title}\n${text}`;
	const hash = sha(source);
	const createdAt = sourceUpdatedAt;
	return {
		id: `${namespace}-search-${resourceType}-${resourceId}`,
		resource_type: resourceType,
		resource_id: resourceId,
		chunk_index: 0,
		title,
		text,
		text_hash: hash,
		source_updated_at: sourceUpdatedAt,
		created_at: createdAt,
		updated_at: createdAt,
	};
}

function insertSyncRow(table, row) {
	return upsert(table, row);
}

function insertJoin(table, ids, syncId, createdAt) {
	return upsert(table, withSync(ids, new Date(createdAt), syncId));
}

function withSync(row, date, syncId = row.id) {
	return {
		...row,
		_sync_id: syncId,
		_updated_at: ms(date),
		_deleted: row.deleted_at ? 1 : 0,
		_extra: json({ seeded: true, namespace }),
		_version: 1,
	};
}

function upsert(table, row) {
	const columns = Object.keys(row);
	return `INSERT OR REPLACE INTO ${table} (${columns.join(', ')}) VALUES (${columns.map(column => sqlValue(row[column])).join(', ')});`;
}

function lexicalDocument(title, paragraphs) {
	const nodes = paragraphs.map(text => ({
		children: [
			{
				detail: 0,
				format: 0,
				mode: 'normal',
				style: '',
				text,
				type: 'text',
				version: 1,
			},
		],
		direction: null,
		format: '',
		indent: 0,
		type: 'paragraph',
		version: 1,
		textFormat: 0,
		textStyle: '',
	}));
	return json({
		root: {
			children: nodes,
			direction: null,
			format: '',
			indent: 0,
			type: 'root',
			version: 1,
		},
		title,
	});
}

function without(row, keys) {
	const copy = { ...row };
	for (const key of keys) delete copy[key];
	return copy;
}

function tagMap(tags) {
	return Object.fromEntries(tags.map(tag => [tag.name, tag.id]));
}

function unique(values) {
	return [...new Set(values.filter(Boolean))];
}

function sqlValue(value) {
	if (value == null) return 'NULL';
	if (typeof value === 'number') return Number.isFinite(value) ? String(value) : 'NULL';
	return `'${String(value).replaceAll("'", "''")}'`;
}

function json(value) {
	return JSON.stringify(value);
}

function sha(value) {
	return createHash('sha256').update(value).digest('hex');
}

function startOfUtcDay(date) {
	return new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()));
}

function addDays(date, amount) {
	const next = new Date(date);
	next.setUTCDate(next.getUTCDate() + amount);
	return next;
}

function addHours(date, amount) {
	const next = new Date(date);
	next.setUTCHours(next.getUTCHours() + amount);
	return next;
}

function addMinutes(date, amount) {
	const next = new Date(date);
	next.setUTCMinutes(next.getUTCMinutes() + amount);
	return next;
}

function iso(date) {
	return date.toISOString();
}

function ms(date) {
	return date.getTime();
}

function dateId(date) {
	return iso(date).slice(0, 10);
}

function formatShort(date) {
	return date.toISOString().slice(0, 10);
}
