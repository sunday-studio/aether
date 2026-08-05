import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import process from 'node:process';

const mobileRoot = resolve(import.meta.dirname, '..');
const expectedVersion = process.argv.slice(2).find((argument) => argument !== '--');

const [packageJson, cargoToml, tauriConfig] = await Promise.all([
	readFile(resolve(mobileRoot, 'package.json'), 'utf8'),
	readFile(resolve(mobileRoot, 'src-tauri/Cargo.toml'), 'utf8'),
	readFile(resolve(mobileRoot, 'src-tauri/tauri.conf.json'), 'utf8'),
]);

const versions = {
	packageJson: JSON.parse(packageJson).version,
	cargoToml: cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1],
	tauriConfig: JSON.parse(tauriConfig).version,
};

if (Object.values(versions).some((version) => !version)) {
	throw new Error(`Could not read every mobile version: ${JSON.stringify(versions)}`);
}

const uniqueVersions = [...new Set(Object.values(versions))];
if (uniqueVersions.length !== 1) {
	throw new Error(`Mobile version mismatch: ${JSON.stringify(versions)}`);
}

if (expectedVersion && uniqueVersions[0] !== expectedVersion) {
	throw new Error(
		`Release version ${expectedVersion} does not match committed mobile version ${uniqueVersions[0]}`,
	);
}

console.log(`Mobile version parity verified: ${uniqueVersions[0]}`);
