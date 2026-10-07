#!/usr/bin/env node
import { fetchGlobalLeaderboard, deleteGlobalScore, clearGlobalLeaderboard } from "../api/leaderboard.js";

const [action, ...args] = process.argv.slice(2);

async function main() {
  if (!action || action === "list" || action === "--list") {
    console.log("\nFetching global leaderboard entries...\n");
    try {
      const scores = await fetchGlobalLeaderboard();
      if (!scores || scores.length === 0) {
        console.log("Leaderboard is currently empty.");
        return;
      }
      console.table(scores.map((s, idx) => ({
        Rank: idx + 1,
        "Display Name": s.display_name,
        "High Score": s.high_score.toLocaleString()
      })));
    } catch (err) {
      console.error("Failed to fetch leaderboard:", err.message);
      process.exit(1);
    }
    return;
  }

  if (action === "delete" || action === "--delete") {
    const target = args.join(" ").trim();
    if (!target) {
      console.error("Usage: node scripts/manage-leaderboard.mjs delete <username>");
      process.exit(1);
    }
    console.log(`\nDeleting score for "${target}" from global leaderboard...`);
    try {
      const updated = await deleteGlobalScore(target);
      console.log(`Successfully deleted "${target}". Remaining entries: ${updated.length}`);
      if (updated.length > 0) {
        console.table(updated.map((s, idx) => ({
          Rank: idx + 1,
          "Display Name": s.display_name,
          "High Score": s.high_score.toLocaleString()
        })));
      }
    } catch (err) {
      console.error("Failed to delete score:", err.message);
      process.exit(1);
    }
    return;
  }

  if (action === "clear" || action === "--clear") {
    console.log("\nResetting entire global leaderboard...");
    try {
      await clearGlobalLeaderboard();
      console.log("Global leaderboard has been cleared completely.");
    } catch (err) {
      console.error("Failed to clear leaderboard:", err.message);
      process.exit(1);
    }
    return;
  }

  console.log(`
GraviPop Leaderboard Manager
----------------------------
Usage:
  node scripts/manage-leaderboard.mjs list
  node scripts/manage-leaderboard.mjs delete <username>
  node scripts/manage-leaderboard.mjs clear
`);
}

main();
