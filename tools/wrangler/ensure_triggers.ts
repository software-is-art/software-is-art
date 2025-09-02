import { execSync } from 'child_process';
import { readFileSync } from 'fs';
import path from 'path';

const args = process.argv.slice(2);
const domainsIndex = args.indexOf('--domains');
const domainsPath = domainsIndex >= 0 ? args[domainsIndex + 1] : 'tools/domains.json';
const domains = JSON.parse(readFileSync(domainsPath, 'utf8'));

for (const moduleName of Object.keys(domains.modules)) {
  for (const env of domains.envs) {
    const cwd = path.join('workers', moduleName);
    const cmd = `npx wrangler triggers deploy --env ${env}`;
    console.log(`[wrangler] ${cmd} (cwd=${cwd})`);
    try {
      execSync(cmd, { stdio: 'inherit', cwd });
    } catch (err) {
      console.error(`Failed to deploy triggers for ${moduleName} (${env})`);
    }
  }
}

if (args.includes('--comment-pr')) {
  const manifest = args[args.indexOf('--comment-pr') + 1];
  console.log(`Would comment PR with manifest: ${manifest}`);
}
