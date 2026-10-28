import type { ComputedRef } from "vue";
export interface FieldContext {
  id: string;
  name?: string;
  required?: boolean;
  error: ComputedRef<string>;
}
