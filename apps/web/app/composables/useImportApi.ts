import type {
  ExpenseImportPreview,
  ImportKind,
  ImportResult,
  IncomeImportPreview,
  RecurringExpenseImportPreview,
  VariableExpenseImportPreview,
} from "~/types/import";

export const importSampleUrls: Record<ImportKind, string> = {
  expense: "/api/import/expenses/sample",
  income: "/api/import/incomes/sample",
  "recurring-expense": "/api/import/recurring-expenses/sample",
  "variable-expense": "/api/import/variable-expenses/sample",
};

const importPaths: Record<ImportKind, string> = {
  expense: "/api/import/expenses",
  income: "/api/import/incomes",
  "recurring-expense": "/api/import/recurring-expenses",
  "variable-expense": "/api/import/variable-expenses",
};

export function useImportApi() {
  const client = $fetch.create({ retry: 0, timeout: 15000 });

  function preview(kind: "expense", csv: string): Promise<ExpenseImportPreview>;
  function preview(kind: "income", csv: string): Promise<IncomeImportPreview>;
  function preview(kind: "recurring-expense", csv: string): Promise<RecurringExpenseImportPreview>;
  function preview(kind: "variable-expense", csv: string): Promise<VariableExpenseImportPreview>;
  function preview(kind: ImportKind, csv: string) {
    return client(`${importPaths[kind]}/preview`, {
      method: "POST",
      headers: { "content-type": "text/csv" },
      body: csv,
    });
  }

  function run(kind: ImportKind, csv: string) {
    return client<ImportResult>(importPaths[kind], {
      method: "POST",
      headers: { "content-type": "text/csv" },
      body: csv,
    });
  }

  return { preview, run };
}
