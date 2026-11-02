import type { ChatMessage } from "./messages";

/** Messages from the same author closer than this are grouped. */
export const GROUP_WINDOW_MS = 7 * 60 * 1000;

export interface MessageRow {
  message: ChatMessage;
  /** Show avatar, name and time (first of a group). */
  showHeader: boolean;
  /** Start of a new calendar day: show a separator above. */
  newDay: Date | null;
}

function sameDay(a: Date, b: Date): boolean {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

export function groupMessages(list: ChatMessage[]): MessageRow[] {
  const rows: MessageRow[] = [];
  let prev: ChatMessage | null = null;
  let prevDate: Date | null = null;
  for (const message of list) {
    const date = new Date(message.createdAt);
    const newDay = !prevDate || !sameDay(prevDate, date) ? date : null;
    const showHeader =
      !prev ||
      newDay !== null ||
      prev.userId !== message.userId ||
      message.userId === null ||
      date.getTime() - (prevDate?.getTime() ?? 0) > GROUP_WINDOW_MS;
    rows.push({ message, showHeader, newDay });
    prev = message;
    prevDate = date;
  }
  return rows;
}

export type DayLabel = "today" | "yesterday" | "date";

export function dayLabel(date: Date, now = new Date()): DayLabel {
  if (sameDay(date, now)) return "today";
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  return sameDay(date, yesterday) ? "yesterday" : "date";
}
