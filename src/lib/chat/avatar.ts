export const AVATAR_TINTS = 7;

/** "Léa Martin" → "LM", "samir" → "SA". */
export function initials(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return "?";
  if (words.length === 1) return Array.from(words[0]).slice(0, 2).join("").toUpperCase();
  return (Array.from(words[0])[0] + Array.from(words[words.length - 1])[0]).toUpperCase();
}

/** Stable tint index for a user id (the API has no avatars). */
export function tintIndex(id: string): number {
  let hash = 0;
  for (let i = 0; i < id.length; i++) hash = (hash * 31 + id.charCodeAt(i)) | 0;
  return Math.abs(hash) % AVATAR_TINTS;
}
