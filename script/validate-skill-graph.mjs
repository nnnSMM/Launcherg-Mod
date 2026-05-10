import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const graphRoot = path.join(root, "docs", "skill-graph");
const requiredKeys = ["id", "title", "type", "status", "updated"];

async function collectMarkdownFiles(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await collectMarkdownFiles(fullPath)));
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(fullPath);
    }
  }

  return files;
}

function parseFrontmatter(content) {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n/);
  if (!match) return null;

  const fields = new Map();
  for (const line of match[1].split(/\r?\n/)) {
    const field = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (field) {
      fields.set(field[1], field[2].replace(/^["']|["']$/g, ""));
    }
  }

  return fields;
}

function findWikiLinks(content) {
  const links = [];
  const wikiLinkPattern = /\[\[([^\]]+)\]\]/g;
  const searchableContent = content
    .replace(/```[\s\S]*?```/g, "")
    .replace(/`[^`\n]*`/g, "");
  let match;

  while ((match = wikiLinkPattern.exec(searchableContent)) !== null) {
    const target = match[1].split("|")[0].split("#")[0].trim();
    if (target.length > 0) {
      links.push(target);
    }
  }

  return links;
}

const files = await collectMarkdownFiles(graphRoot);
const targets = new Set();
const notes = [];
const errors = [];

for (const file of files) {
  const content = await readFile(file, "utf8");
  const relativePath = path.relative(graphRoot, file).replaceAll(path.sep, "/");
  const frontmatter = parseFrontmatter(content);

  if (!frontmatter) {
    errors.push(`${relativePath}: YAML frontmatter is missing.`);
    continue;
  }

  for (const key of requiredKeys) {
    if (!frontmatter.has(key) || frontmatter.get(key).trim() === "") {
      errors.push(`${relativePath}: required frontmatter key "${key}" is missing.`);
    }
  }

  const updated = frontmatter.get("updated");
  if (updated && !/^\d{4}-\d{2}-\d{2}$/.test(updated)) {
    errors.push(`${relativePath}: updated must use YYYY-MM-DD.`);
  }

  const id = frontmatter.get("id");
  if (id) {
    targets.add(id);
  }
  targets.add(path.basename(file, ".md"));
  notes.push({ relativePath, content });
}

for (const note of notes) {
  for (const link of findWikiLinks(note.content)) {
    if (!targets.has(link)) {
      errors.push(`${note.relativePath}: unresolved wikilink [[${link}]].`);
    }
  }
}

if (errors.length > 0) {
  console.error(`Skill Graph validation failed with ${errors.length} error(s):`);
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

console.log(`Skill Graph validation passed: ${files.length} markdown files checked.`);
