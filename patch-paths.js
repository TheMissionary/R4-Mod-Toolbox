import fs from 'node:fs';

const filePath = 'build/index.html';
if (fs.existsSync(filePath)) {
  let content = fs.readFileSync(filePath, 'utf8');
  content = content.replaceAll('href="/_app/', 'href="./_app/');
  content = content.replaceAll('src="/_app/', 'src="./_app/');
  content = content.replaceAll('import("/_app/', 'import("./_app/');
  content = content.replaceAll('href="/favicon.png"', 'href="./favicon.png"');
  fs.writeFileSync(filePath, content, 'utf8');
  console.log('[patch-paths] Successfully rewrote build/index.html to document-relative paths.');
} else {
  console.error('[patch-paths] Error: build/index.html not found.');
  process.exit(1);
}