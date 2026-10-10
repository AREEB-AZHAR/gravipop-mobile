#!/usr/bin/env node
import { execSync } from "node:child_process";
import { readFileSync, existsSync } from "node:fs";

// Patterns that identify sensitive keys and credentials
const SECRET_PATTERNS = [
  {
    name: "Google / Firebase API Key",
    regex: /AIza[0-9A-Za-z\-_]{35}/g,
  },
  {
    name: "Dreamlo Private Key",
    regex: /BKaONk[0-9A-Za-z\-_]{30,}/g,
  },
  {
    name: "Hardcoded API Key Assignment",
    regex: /(?:apiKey|api_key|privateKey|private_key|secretKey|secret_key)\s*[:=]\s*["'][A-Za-z0-9\-_]{20,}["']/gi,
  },
  {
    name: "Private Key Header",
    regex: /-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----/g,
  },
];

// Files that must never be tracked or staged (actual env files with secrets)
const FORBIDDEN_FILE_PATTERNS = [
  /^\.env$/i,
  /^\.env\.(local|development|production|staging|test)/i,
  /\.pem$/i,
  /\.key$/i,
  /id_rsa/i,
  /google-services\.json$/i,
  /GoogleService-Info\.plist$/i,
];

function getStagedFiles() {
  try {
    const output = execSync("git diff --cached --name-only --diff-filter=ACM", { encoding: "utf8" });
    return output.split("\n").map(f => f.trim()).filter(Boolean);
  } catch (_) {
    return [];
  }
}

function getStagedDiff(file) {
  try {
    return execSync(`git diff --cached -- "${file}"`, { encoding: "utf8" });
  } catch (_) {
    return "";
  }
}

function main() {
  console.log("🔒 Running automated pre-commit secret detection...");
  const stagedFiles = getStagedFiles();

  if (stagedFiles.length === 0) {
    console.log("No staged files to scan.");
    process.exit(0);
  }

  let violations = [];

  for (const file of stagedFiles) {
    const basename = file.split("/").pop()?.split("\\").pop() || file;
    // Allow documented template files like .env.example
    if (basename.toLowerCase() === ".env.example") {
      continue;
    }

    // 1. Check for forbidden file names
    for (const pattern of FORBIDDEN_FILE_PATTERNS) {
      if (pattern.test(basename)) {
        violations.push({
          file,
          rule: "Forbidden Sensitive File",
          match: basename,
          line: "Entire file must not be committed",
        });
      }
    }

    // Skip scanning check-secrets itself or template examples
    if (file.endsWith("check-secrets.mjs") || file.endsWith(".env.example")) {
      continue;
    }

    // 2. Check staged diff content additions
    const diff = getStagedDiff(file);
    const addedLines = diff.split("\n").filter(l => l.startsWith("+") && !l.startsWith("+++"));

    for (let i = 0; i < addedLines.length; i++) {
      const line = addedLines[i].substring(1); // Strip leading '+'
      for (const pattern of SECRET_PATTERNS) {
        pattern.regex.lastIndex = 0;
        const match = pattern.regex.exec(line);
        if (match) {
          violations.push({
            file,
            rule: pattern.name,
            match: match[0],
            line: `Line addition: ${line.trim().substring(0, 80)}`,
          });
        }
      }
    }
  }

  if (violations.length > 0) {
    console.error("\n❌ PRE-COMMIT SECURITY CHECK FAILED!");
    console.error("Exposed secret or sensitive file detected in staged changes:\n");
    for (const v of violations) {
      console.error(`  • [${v.rule}] in ${v.file}`);
      console.error(`    Details: ${v.line}`);
      console.error(`    Match:   ${v.match.substring(0, 10)}... (masked)\n`);
    }
    console.error("Please move all API keys and secrets to .env and stage only environment variable references.");
    console.error("Commit aborted.\n");
    process.exit(1);
  }

  console.log("✅ Secret check passed: No exposed API keys or sensitive files staged.\n");
  process.exit(0);
}

main();
