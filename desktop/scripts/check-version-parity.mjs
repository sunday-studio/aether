import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import process from 'node:process';

const desktopRoot = resolve(import.meta.dirname, '..');
const expectedVersion = process.argv.slice(2).find(argument => argument !== '--');

const [packageJson, cargoToml, tauriConfig] = await Promise.all([
	readFile(resolve(desktopRoot, 'package.json'), 'utf8'),
	readFile(resolve(desktopRoot, 'src-tauri/Cargo.toml'), 'utf8'),
	readFile(resolve(desktopRoot, 'src-tauri/tauri.conf.json'), 'utf8'),
]);
const parsedTauriConfig = JSON.parse(tauriConfig);

const versions = {
	packageJson: JSON.parse(packageJson).version,
	cargoToml: cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1],
	tauriConfig: parsedTauriConfig.version,
};

if (Object.values(versions).some(version => !version)) {
	throw new Error(`Could not read every desktop version: ${JSON.stringify(versions)}`);
}

const uniqueVersions = [...new Set(Object.values(versions))];
if (uniqueVersions.length !== 1) {
	throw new Error(`Desktop version mismatch: ${JSON.stringify(versions)}`);
}

if (expectedVersion && uniqueVersions[0] !== expectedVersion) {
	throw new Error(
		`Release version ${expectedVersion} does not match committed desktop version ${uniqueVersions[0]}`,
	);
}

if (parsedTauriConfig.bundle.createUpdaterArtifacts !== true) {
	throw new Error('Tauri must enable bundle.createUpdaterArtifacts for signed updater archives');
}

const updaterEndpoint = parsedTauriConfig.plugins?.updater?.endpoints?.[0];
if (updaterEndpoint !== 'https://sunday-studio.github.io/aether/updates/desktop/stable/latest.json') {
	throw new Error(`Unexpected desktop updater endpoint: ${updaterEndpoint ?? 'missing'}`);
}

console.log(`Desktop version parity verified: ${uniqueVersions[0]}`);
