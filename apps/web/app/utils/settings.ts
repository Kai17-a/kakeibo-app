interface OrderedCategory {
  id: string;
  parent_category_id: string | null;
  display_order: number;
}
export function groupCategories<T extends OrderedCategory>(items: T[]): T[] {
  const sorted = [...items].sort((a, b) => a.display_order - b.display_order);
  return sorted
    .filter((item) => item.parent_category_id === null)
    .flatMap((parent) => [
      parent,
      ...sorted.filter((item) => item.parent_category_id === parent.id),
    ]);
}
export function categoryMove<T extends OrderedCategory>(items: T[], id: string, direction: -1 | 1) {
  const category = items.find((item) => item.id === id);
  if (!category) return null;
  const siblings = groupCategories(items).filter(
    (item) => item.parent_category_id === category.parent_category_id,
  );
  const index = siblings.findIndex((item) => item.id === id);
  const target = index + direction;
  const current = siblings[index];
  const neighbor = siblings[target];
  if (!current || !neighbor) return null;
  siblings[index] = neighbor;
  siblings[target] = current;
  const positions = new Map(siblings.map((item, order) => [item.id, order]));
  return {
    input: {
      parent_category_id: category.parent_category_id,
      category_ids: siblings.map((item) => item.id),
    },
    items: items.map((item) => ({
      ...item,
      display_order: positions.get(item.id) ?? item.display_order,
    })),
  };
}
export function availableBudgetCategories<T extends { id: string }>(
  categories: T[],
  budgets: { category_id: string }[],
  editingCategoryId?: string,
) {
  return categories.filter(
    (category) =>
      category.id === editingCategoryId ||
      !budgets.some((budget) => budget.category_id === category.id),
  );
}
export function validateNamed(state: { name: string; description: string }) {
  const errors: { name: string; message: string }[] = [];
  if (!state.name.trim()) errors.push({ name: "name", message: "名前を入力してください。" });
  if (state.name.trim().length > 100)
    errors.push({ name: "name", message: "100文字以内で入力してください。" });
  if (state.description.length > 500)
    errors.push({ name: "description", message: "500文字以内で入力してください。" });
  return errors;
}
export function validAmount(value: string | number): boolean {
  return (
    String(value).trim() !== "" &&
    Number.isFinite(Number(value)) &&
    Number(value) >= 0 &&
    Number.isInteger(Number(value))
  );
}
export function formatYen(value: string | null): string {
  return value === null ? "未設定" : `${Number(value).toLocaleString("ja-JP")}円`;
}
