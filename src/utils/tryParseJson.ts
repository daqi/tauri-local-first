export default function tryParseJSON(v: any) {
  if (typeof v !== 'string') return v;
  const s = v.trim();
  if (
    (s.startsWith('{') && s.endsWith('}')) ||
    (s.startsWith('[') && s.endsWith(']'))
  ) {
    try {
      return JSON.parse(s);
    } catch (_) {
      return v;
    }
  }
  return v;
}
