export type ImportKind = "expense" | "income" | "recurring-expense";

export interface ImportResult {
  imported: number;
  created_categories: string[];
  created_payment_methods: string[];
}

export interface ExpensePreviewRow {
  transaction_date: string;
  amount: string;
  category: string;
  category_is_new: boolean;
  payment_method: string;
  payment_method_is_new: boolean;
  description: string | null;
}
export interface ExpenseImportPreview {
  rows: ExpensePreviewRow[];
  created_categories: string[];
  created_payment_methods: string[];
}

export interface IncomePreviewRow {
  transaction_date: string;
  amount: string;
  category: string;
  category_is_new: boolean;
  description: string | null;
}
export interface IncomeImportPreview {
  rows: IncomePreviewRow[];
  created_categories: string[];
}

export interface RecurringExpensePreviewRow {
  name: string;
  amount: string;
  currency: string;
  foreign_amount: string;
  payment_day: string;
  start_date: string;
  end_date: string | null;
  category: string;
  category_is_new: boolean;
  payment_method: string;
  payment_method_is_new: boolean;
  is_variable: string;
  description: string | null;
}
export interface RecurringExpenseImportPreview {
  rows: RecurringExpensePreviewRow[];
  created_categories: string[];
  created_payment_methods: string[];
}

export type ImportPreview =
  | ExpenseImportPreview
  | IncomeImportPreview
  | RecurringExpenseImportPreview;
