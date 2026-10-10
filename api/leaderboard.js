if (typeof process.loadEnvFile === "function") {
  try {
    process.loadEnvFile(".env");
  } catch (_) {}
}

const DREAMLO_PUBLIC = process.env.DREAMLO_PUBLIC_CODE || process.env.VITE_DREAMLO_PUBLIC_CODE || "";
const DREAMLO_PRIVATE = process.env.DREAMLO_PRIVATE_KEY || "";

export async function fetchGlobalLeaderboard() {
  if (!DREAMLO_PUBLIC) {
    console.warn("Dreamlo public code is not configured in environment variables");
    return [];
  }
  try {
    const res = await fetch(`http://dreamlo.com/lb/${DREAMLO_PUBLIC}/json`, {
      headers: { "Accept": "application/json" }, signal: AbortSignal.timeout(5000)
    });
    if (!res.ok) {
      throw new Error(`Leaderboard provider unavailable (HTTP ${res.status})`);
    }
    const data = await res.json();
    const rawEntries = data?.dreamlo?.leaderboard?.entry;
    if (!rawEntries) {
      return [];
    }
    const list = Array.isArray(rawEntries) ? rawEntries : [rawEntries];

    // Deduplicate by username (preserving exact casing for distinct names), keeping each player's highest score
    const byName = new Map();
    for (const item of list) {
      if (!item || !item.name || typeof item.score === "undefined") continue;
      const displayName = decodeURIComponent(item.name).replace(/\+/g, " ").trim();
      const score = Math.max(0, parseInt(item.score, 10) || 0);
      const key = displayName;
      if (!byName.has(key) || score > byName.get(key).high_score) {
        byName.set(key, { display_name: displayName, high_score: score });
      }
    }

    const normalized = Array.from(byName.values())
      .sort((a, b) => b.high_score - a.high_score)
      .slice(0, 20);

    return normalized;
  } catch (err) {
    console.error("Leaderboard fetch error:", err);
    throw new Error("Leaderboard provider unavailable");
  }
}

export async function submitGlobalScore(name, score) {
  const cleanName = (name || "").replace(/[^\w\s.-]/g, "").trim().substring(0, 20);
  const cleanScore = Math.max(0, Math.min(1000000000, parseInt(score, 10) || 0));

  if (!cleanName || cleanName.length < 3) {
    throw new Error("Display name must be between 3 and 20 characters");
  }

  if (!DREAMLO_PRIVATE) {
    throw new Error("Leaderboard private key is not configured on server (DREAMLO_PRIVATE_KEY)");
  }

  // 1. Try-catch to check for existing username entries on the leaderboard (exact case-match)
  try {
    const existingScores = await fetchGlobalLeaderboard();
    const existingEntry = existingScores.find(
      entry => entry.display_name === cleanName
    );

    if (existingEntry) {
      // If the existing entry already has an equal or higher score, keep it (do not downgrade)
      if (cleanScore <= existingEntry.high_score) {
        return existingScores;
      }

      // If the new score is higher, delete the previous entry to prevent duplicate records
      const oldEncoded = encodeURIComponent(existingEntry.display_name);
      await fetch(`http://dreamlo.com/lb/${DREAMLO_PRIVATE}/delete/${oldEncoded}`, {
        signal: AbortSignal.timeout(3000)
      }).catch(() => {});
    }
  } catch (lookupErr) {
    console.warn("Existing username lookup check failed, continuing with direct submission:", lookupErr);
  }

  // 2. Submit new score to Dreamlo
  const encodedName = encodeURIComponent(cleanName);
  const res = await fetch(`http://dreamlo.com/lb/${DREAMLO_PRIVATE}/add/${encodedName}/${cleanScore}`, {
    signal: AbortSignal.timeout(5000)
  });
  if (!res.ok) {
    throw new Error(`Failed to record score (HTTP ${res.status})`);
  }

  return await fetchGlobalLeaderboard();
}

export async function deleteGlobalScore(name) {
  const cleanName = (name || "").replace(/[^\w\s.-]/g, "").trim().substring(0, 20);
  if (!cleanName) {
    throw new Error("Display name is required for deletion");
  }

  if (!DREAMLO_PRIVATE) {
    throw new Error("Leaderboard private key is not configured on server (DREAMLO_PRIVATE_KEY)");
  }

  // Also remove exact matching entries from Dreamlo
  try {
    const existingScores = await fetchGlobalLeaderboard();
    const matches = existingScores.filter(
      entry => entry.display_name === cleanName
    );
    for (const m of matches) {
      const enc = encodeURIComponent(m.display_name);
      await fetch(`http://dreamlo.com/lb/${DREAMLO_PRIVATE}/delete/${enc}`, {
        signal: AbortSignal.timeout(3000)
      }).catch(() => {});
    }
  } catch (_) {}

  const encodedName = encodeURIComponent(cleanName);
  const res = await fetch(`http://dreamlo.com/lb/${DREAMLO_PRIVATE}/delete/${encodedName}`, {
    signal: AbortSignal.timeout(5000)
  });
  if (!res.ok) {
    throw new Error(`Failed to delete score (HTTP ${res.status})`);
  }

  return await fetchGlobalLeaderboard();
}

export async function clearGlobalLeaderboard() {
  if (!DREAMLO_PRIVATE) {
    throw new Error("Leaderboard private key is not configured on server (DREAMLO_PRIVATE_KEY)");
  }

  const res = await fetch(`http://dreamlo.com/lb/${DREAMLO_PRIVATE}/clear`, {
    signal: AbortSignal.timeout(5000)
  });
  if (!res.ok) {
    throw new Error(`Failed to clear leaderboard (HTTP ${res.status})`);
  }
  return [];
}

export default async function handler(req, res) {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  if (req.method === "OPTIONS") {
    res.statusCode = 204;
    res.end();
    return;
  }

  if (req.method === "GET") {
    let scores;
    try { scores = await fetchGlobalLeaderboard(); }
    catch (_) {
      res.setHeader("Content-Type", "application/json");
      res.setHeader("Cache-Control", "no-store");
      res.statusCode = 503;
      res.end(JSON.stringify({ error: "Leaderboard unavailable. Try again later." }));
      return;
    }
    res.setHeader("Content-Type", "application/json");
    res.setHeader("Cache-Control", "public, s-maxage=3, stale-while-revalidate=10");
    res.statusCode = 200;
    res.end(JSON.stringify(scores));
    return;
  }

  if (req.method === "POST") {
    let body = req.body;
    if (typeof body === "string") {
      try {
        body = JSON.parse(body);
      } catch (_) {
        body = {};
      }
    }
    const name = body?.display_name || body?.name;
    const score = body?.high_score ?? body?.score;

    try {
      const updatedScores = await submitGlobalScore(name, score);
      res.setHeader("Content-Type", "application/json");
      res.statusCode = 200;
      res.end(JSON.stringify({ success: true, scores: updatedScores }));
    } catch (err) {
      res.setHeader("Content-Type", "application/json");
      res.statusCode = 400;
      res.end(JSON.stringify({ error: err.message || "Invalid score submission" }));
    }
    return;
  }

  if (req.method === "DELETE") {
    let name = null;
    let isClear = false;

    if (req.url && req.url.includes("?")) {
      const qs = new URLSearchParams(req.url.split("?")[1]);
      name = qs.get("name");
      isClear = qs.get("clear") === "all" || qs.get("clear") === "true";
    }

    let body = req.body;
    if (typeof body === "string") {
      try { body = JSON.parse(body); } catch (_) { body = {}; }
    }
    if (body) {
      name = name || body.name || body.display_name;
      if (body.clear === "all" || body.clear === true) {
        isClear = true;
      }
    }

    try {
      if (isClear) {
        await clearGlobalLeaderboard();
        res.setHeader("Content-Type", "application/json");
        res.statusCode = 200;
        res.end(JSON.stringify({ success: true, message: "Leaderboard cleared successfully." }));
      } else if (name) {
        const updatedScores = await deleteGlobalScore(name);
        res.setHeader("Content-Type", "application/json");
        res.statusCode = 200;
        res.end(JSON.stringify({ success: true, message: `Score for "${name}" deleted successfully.`, scores: updatedScores }));
      } else {
        res.setHeader("Content-Type", "application/json");
        res.statusCode = 400;
        res.end(JSON.stringify({ error: "Specify a 'name' parameter to delete or 'clear=all' to reset." }));
      }
    } catch (err) {
      res.setHeader("Content-Type", "application/json");
      res.statusCode = 500;
      res.end(JSON.stringify({ error: err.message || "Failed to delete score" }));
    }
    return;
  }

  res.statusCode = 405;
  res.end("Method Not Allowed");
}
