import { describe, expect, it } from "vitest";
import type { Message } from "$lib/bindings/Message";
import { initials, tintIndex, AVATAR_TINTS } from "./avatar";
import { dayLabel, groupMessages } from "./grouping";
import { confirm, fail, isEdited, makeLocal, mergeHistory, removeById, retrying, upsert, type ChatMessage } from "./messages";
import { applyBroadcast, fromSnapshot, statusOf } from "./presence";
import { validateCredentials } from "./validation";

function msg(id: string, userId: string | null, createdAt: string, content = id): Message {
  return { id, userId, channelId: "c1", content, createdAt, updatedAt: createdAt };
}

describe("message list", () => {
  it("inserts in order and replaces by id", () => {
    let list: ChatMessage[] = [];
    list = upsert(list, msg("b", "u1", "2026-11-02T10:00:02Z"));
    list = upsert(list, msg("a", "u1", "2026-11-02T10:00:01Z"));
    list = upsert(list, msg("c", "u1", "2026-11-02T10:00:03Z"));
    list = upsert(list, { ...msg("b", "u1", "2026-11-02T10:00:02Z"), content: "edited" });
    expect(list.map((m) => m.id)).toEqual(["a", "b", "c"]);
    expect(list[1].content).toBe("edited");
  });

  it("keeps pending messages at the bottom", () => {
    const local = makeLocal("c1", "me", "hi", new Date("2026-11-02T09:00:00Z"));
    let list: ChatMessage[] = upsert([], msg("a", "u1", "2026-11-02T10:00:01Z"));
    list = [...list, local];
    list = upsert(list, msg("b", "u2", "2026-11-02T10:00:05Z"));
    expect(list.map((m) => m.id)).toEqual(["a", "b", local.id]);
  });

  it("confirms a local message without duplicating the broadcast", () => {
    const local = makeLocal("c1", "me", "hi");
    const server = msg("m1", "me", "2026-11-02T10:00:00Z", "hi");
    // Broadcast arrived first, then the command resolved.
    let list = upsert([local], server);
    list = confirm(list, local.id, server);
    expect(list).toEqual([server]);
  });

  it("marks failures and retries", () => {
    const local = makeLocal("c1", "me", "hi");
    const other = msg("a", "u1", "2026-11-02T10:00:00Z");
    let list: ChatMessage[] = [local, other];
    list = fail(list, local.id, { code: "DISCONNECTED", status: null, message: "" });
    expect(list[0].local?.status).toBe("failed");
    list = retrying(list, local.id);
    expect(list.map((m) => m.id)).toEqual(["a", local.id]);
    expect(list[1].local?.status).toBe("pending");
    expect(removeById(list, local.id)).toEqual([other]);
  });

  it("keeps in-flight messages across a reload", () => {
    const local = makeLocal("c1", "me", "hi");
    const history = [msg("a", "u1", "2026-11-02T10:00:00Z")];
    const stale = msg("old", "u1", "2026-11-01T10:00:00Z");
    expect(mergeHistory(history, [stale, local], new Set(["old"]))).toEqual([...history, local]);
  });

  it("keeps messages received while the history was loading", () => {
    const history = [msg("a", "u1", "2026-11-02T10:00:00Z")];
    const live = msg("b", "u2", "2026-11-02T10:00:09Z");
    expect(mergeHistory(history, [live], new Set()).map((m) => m.id)).toEqual(["a", "b"]);
  });

  it("detects edits", () => {
    expect(isEdited(msg("a", "u", "2026-11-02T10:00:00Z"))).toBe(false);
    expect(isEdited({ ...msg("a", "u", "2026-11-02T10:00:00Z"), updatedAt: "2026-11-02T10:05:00Z" })).toBe(true);
  });
});

describe("grouping", () => {
  it("groups consecutive messages of the same author within 7 minutes", () => {
    const rows = groupMessages([
      msg("1", "u1", "2026-11-02T10:00:00Z"),
      msg("2", "u1", "2026-11-02T10:06:00Z"),
      msg("3", "u1", "2026-11-02T10:14:00Z"),
      msg("4", "u2", "2026-11-02T10:14:30Z"),
      msg("5", null, "2026-11-02T10:15:00Z"),
      msg("6", null, "2026-11-02T10:15:10Z"),
    ]);
    expect(rows.map((r) => r.showHeader)).toEqual([true, false, true, true, true, true]);
  });

  it("starts a new group and a separator on a new day", () => {
    const a = new Date(2026, 10, 1, 23, 58).toISOString();
    const b = new Date(2026, 10, 2, 0, 1).toISOString();
    const rows = groupMessages([msg("1", "u1", a), msg("2", "u1", b)]);
    expect(rows[0].newDay).not.toBeNull();
    expect(rows[1].newDay).not.toBeNull();
    expect(rows[1].showHeader).toBe(true);
  });

  it("labels days", () => {
    const now = new Date(2026, 10, 2, 12);
    expect(dayLabel(new Date(2026, 10, 2, 1), now)).toBe("today");
    expect(dayLabel(new Date(2026, 10, 1, 23), now)).toBe("yesterday");
    expect(dayLabel(new Date(2026, 9, 30), now)).toBe("date");
  });
});

describe("presence", () => {
  it("builds from the snapshot and applies broadcasts", () => {
    let view = fromSnapshot(
      { presences: [{ userId: "me", status: "online", invisible: true }, { userId: "u1", status: "idle" }] },
      "me",
    );
    expect(view.invisible).toBe(true);
    expect(statusOf(view, "u1")).toBe("idle");
    expect(statusOf(view, "u2")).toBe("offline");

    view = applyBroadcast(view, { userId: "u2", status: "online" }, "me");
    view = applyBroadcast(view, { userId: "me", status: "online", invisible: false }, "me");
    expect(statusOf(view, "u2")).toBe("online");
    expect(view.invisible).toBe(false);
  });
});

describe("avatars", () => {
  it("computes initials", () => {
    expect(initials("Léa Martin")).toBe("LM");
    expect(initials("samir")).toBe("SA");
    expect(initials("  Jean  Paul  Sartre ")).toBe("JS");
    expect(initials("")).toBe("?");
  });

  it("picks a stable tint", () => {
    expect(tintIndex("abc")).toBe(tintIndex("abc"));
    expect(tintIndex("a-very-long-uuid-0000-1111")).toBeLessThan(AVATAR_TINTS);
  });
});

describe("credential validation", () => {
  const ok = { serverUrl: "chat.example.org", username: "Léa", email: "lea@example.org", password: "longenough" };

  it("accepts valid values", () => {
    expect(validateCredentials(ok, "register")).toEqual({});
    expect(validateCredentials({ ...ok, password: "x" }, "login")).toEqual({});
  });

  it("mirrors the server rules on sign up", () => {
    expect(validateCredentials({ ...ok, serverUrl: " ", username: "L", email: "nope", password: "short" }, "register")).toEqual({
      serverUrl: "validation.serverRequired",
      username: "validation.usernameLength",
      email: "validation.emailInvalid",
      password: "validation.passwordLength",
    });
  });
});
