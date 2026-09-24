/**
 * Categorical series colors for charts, distinguishable from each other and from the primary
 * (income) and error (expense) colors, with a lighter variant for dark mode.
 */
export const categoricalChartColors = [
  "light-dark(#2563eb, #60a5fa)",
  "light-dark(#ea580c, #fb923c)",
  "light-dark(#7c3aed, #a78bfa)",
  "light-dark(#0891b2, #22d3ee)",
  "light-dark(#e11d48, #fb7185)",
  "light-dark(#ca8a04, #facc15)",
  "light-dark(#4f46e5, #818cf8)",
  "light-dark(#0f766e, #2dd4bf)",
];

/** The color of the `index`-th series, cycling through the palette. */
export function seriesColor(index: number) {
  return categoricalChartColors[index % categoricalChartColors.length];
}
